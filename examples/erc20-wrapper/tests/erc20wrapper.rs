#![cfg(feature = "e2e")]

use abi::{Erc20, Erc20Wrapper, SafeErc20};
use alloy::{
    primitives::{uint, Address, U256},
    sol,
};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;

use crate::Erc20WrapperExample::constructorCall;

mod abi;
mod mock;

use mock::{erc20, erc20::ERC20Mock};

sol!("src/constructor.sol");

const DECIMALS: u8 = 18;

fn ctr(asset_addr: Address) -> constructorCall {
    Erc20WrapperExample::constructorCall {
        underlyingToken_: asset_addr,
        decimals_: DECIMALS,
    }
}

/// Deploy a new [`Erc20`] contract and [`Erc20Wrapper`] contract and mint
/// initial ERC-20 tokens to `account`.
async fn deploy(
    account: &Account,
    initial_tokens: U256,
) -> Result<(Address, Address)> {
    let asset_addr = erc20::deploy(&account.wallet).await?;

    let contract_addr = account
        .as_deployer()
        .with_constructor(ctr(asset_addr))
        .deploy()
        .await?
        .address()?;

    if initial_tokens > U256::ZERO {
        let asset = ERC20Mock::new(asset_addr, &account.wallet);
        watch!(asset.mint(account.address(), initial_tokens))?;
    }

    Ok((contract_addr, asset_addr))
}

// ============================================================================
// Integration Tests: ERC-20 Wrapper
// ============================================================================

mod constructor {
    use super::*;

    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let asset_address = erc20::deploy(&alice.wallet).await?;
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(asset_address))
            .deploy()
            .await?
            .address()?;
        let contract = Erc20Wrapper::new(contract_addr, alice.wallet);

        let underlying = contract.underlying().call().await?.underlying;
        assert_eq!(underlying, asset_address);

        let decimals = contract.decimals().call().await?.decimals;
        assert_eq!(decimals, DECIMALS);

        Ok(())
    }
}

mod deposit_for {
    use super::*;

    #[e2e::test]
    async fn reverts_when_invalid_asset(alice: Account) -> Result<()> {
        let invalid_asset = alice.address();
        let contract_addr = alice
            .as_deployer()
            .with_constructor(ctr(invalid_asset))
            .deploy()
            .await?
            .address()?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

        let err = send!(contract.depositFor(invalid_asset, uint!(10_U256)))
            .expect_err("should return `InvalidAsset`");
        assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
            token: invalid_asset
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_invalid_receiver(alice: Account) -> Result<()> {
        let initial_supply = uint!(1000_U256);
        let (contract_addr, _) = deploy(&alice, initial_supply).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let amount = uint!(1_U256);

        let err = contract
            .depositFor(contract_addr, amount)
            .call()
            .await
            .expect_err("should return `InvalidReceiver`");

        assert!(err.reverted_with(Erc20Wrapper::ERC20InvalidReceiver {
            receiver: contract_addr
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_insufficient_allowance(alice: Account) -> Result<()> {
        let initial_supply = uint!(1000_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_supply).await?;
        let alice_addr: Address = alice.address();

        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .depositFor(alice_addr, initial_supply)
            .call()
            .await
            .expect_err("should not transfer when insufficient allowance");

        assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
            token: asset_addr
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_insufficient_balance(alice: Account) -> Result<()> {
        let initial_supply = uint!(1000_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_supply).await?;
        let alice_addr: Address = alice.address();

        let value = initial_supply + uint!(1_U256);
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        watch!(asset.approve(contract_addr, value))?;

        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .depositFor(alice_addr, value)
            .call()
            .await
            .expect_err("should not transfer when insufficient balance");

        assert!(err.reverted_with(SafeErc20::SafeErc20FailedOperation {
            token: asset_addr
        }));

        Ok(())
    }

    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let initial_supply = uint!(1000_U256);
        let (contract_addr, asset_addr) =
            deploy(&alice, initial_supply).await?;
        let alice_address = alice.address();
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

        watch!(asset.approve(contract_addr, initial_supply))?;

        let initial_wrapped_balance =
            contract.balanceOf(alice_address).call().await?.balance;
        let initial_wrapped_supply =
            contract.totalSupply().call().await?.totalSupply;

        let value = initial_supply;
        let receipt = receipt!(contract.depositFor(alice_address, value))?;

        // `Transfer` event for ERC-20 token transfer from Alice to the
        // [`Erc20Wrapper`] contract should be emitted.
        assert!(receipt.emits(Erc20::Transfer {
            from: alice_address,
            to: contract_addr,
            value
        }));

        // `Transfer` event for ERC-20 Wrapped token should be emitted (minting
        // wrapped tokens to Alice).
        assert!(receipt.emits(Erc20::Transfer {
            from: Address::ZERO,
            to: alice_address,
            value
        }));

        let wrapped_balance =
            contract.balanceOf(alice_address).call().await?.balance;
        assert_eq!(initial_wrapped_balance + value, wrapped_balance);

        let wrapped_supply = contract.totalSupply().call().await?.totalSupply;
        assert_eq!(initial_wrapped_supply + value, wrapped_supply);

        Ok(())
    }
}

mod withdraw_to {

    use super::*;

    /// Deploy a new [`Erc20`] contract and [`Erc20Wrapper`] contract, mint
    /// initial ERC-20 tokens to `account`, and deposit for `account` to get
    /// wrapped tokens.
    async fn deploy_and_deposit_for(
        account: &Account,
        initial_tokens: U256,
    ) -> Result<(Address, Address)> {
        let (contract_addr, asset_addr) =
            deploy(account, initial_tokens).await?;

        let asset = ERC20Mock::new(asset_addr, &account.wallet);
        let contract = Erc20Wrapper::new(contract_addr, &account.wallet);

        watch!(asset.approve(contract_addr, initial_tokens))?;

        watch!(contract.depositFor(account.address(), initial_tokens))?;

        Ok((contract_addr, asset_addr))
    }

    #[e2e::test]
    async fn reverts_when_invalid_sender(alice: Account) -> Result<()> {
        let initial_tokens = uint!(1000_U256);
        let (contract_addr, _) =
            deploy_and_deposit_for(&alice, initial_tokens).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

        let err = contract
            .withdrawTo(contract_addr, initial_tokens)
            .call()
            .await
            .expect_err("should return `InvalidReciver`");

        assert!(err.reverted_with(Erc20Wrapper::ERC20InvalidReceiver {
            receiver: contract_addr
        }));

        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_insufficient_balance(alice: Account) -> Result<()> {
        let initial_tokens = uint!(1000_U256);
        let (contract_addr, _) =
            deploy_and_deposit_for(&alice, initial_tokens).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

        let value = initial_tokens + uint!(100_U256);

        let wrapped_balance =
            contract.balanceOf(alice.address()).call().await?.balance;

        let err = contract
            .withdrawTo(alice.address(), value)
            .call()
            .await
            .expect_err("should return `InsufficientBalance`");

        assert!(err.reverted_with(Erc20::ERC20InsufficientBalance {
            sender: alice.address(),
            balance: wrapped_balance,
            needed: value
        }));

        Ok(())
    }

    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let initial_tokens = uint!(1000_U256);
        let (contract_addr, _) =
            deploy_and_deposit_for(&alice, initial_tokens).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

        let initial_wrapped_balance =
            contract.balanceOf(alice.address()).call().await?.balance;
        assert_eq!(initial_tokens, initial_wrapped_balance);

        let initial_wrapped_supply =
            contract.totalSupply().call().await?.totalSupply;

        let value = uint!(10_U256);
        let receipt = receipt!(contract.withdrawTo(alice.address(), value))?;

        // `Transfer` event for ERC-20 Wrapped token should be emitted (burning
        // wrapped tokens from Alice).
        assert!(receipt.emits(Erc20::Transfer {
            from: alice.address(),
            to: Address::ZERO,
            value
        }));

        // `Transfer` event for ERC-20 token transfer from
        // [`Erc20Wrapper`] contract to Alice should be emitted.
        assert!(receipt.emits(Erc20::Transfer {
            from: contract_addr,
            to: alice.address(),
            value
        }));

        let wrapped_balance =
            contract.balanceOf(alice.address()).call().await?.balance;
        assert_eq!(initial_wrapped_balance - value, wrapped_balance);

        let wrapped_supply = contract.totalSupply().call().await?.totalSupply;
        assert_eq!(initial_wrapped_supply - value, wrapped_supply);

        Ok(())
    }
}
