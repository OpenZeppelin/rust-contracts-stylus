#![cfg(feature = "e2e")]

use abi::Erc20Wrapper;
use alloy::{
    primitives::{uint, Address, U256},
    sol,
};
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
    Revert,
};
use eyre::Result;

use crate::Erc20WrapperExample::constructorCall;

mod abi;
mod mock;

use mock::{erc20, erc20::ERC20Mock};

sol!("src/constructor.sol");

const WRAPPED_TOKEN_NAME: &str = "WRAPPED Test Token";
const WRAPPED_TOKEN_SYMBOL: &str = "WTTK";

fn ctr(asset_addr: Address) -> constructorCall {
    Erc20WrapperExample::constructorCall {
        underlyingToken_: asset_addr,
        name_: WRAPPED_TOKEN_NAME.to_owned(),
        symbol_: WRAPPED_TOKEN_SYMBOL.to_owned(),
    }
}

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

    // Mint initial tokens to the vault
    if initial_tokens > U256::ZERO {
        let asset = ERC20Mock::new(asset_addr, &account.wallet);
        _ = watch!(asset.mint(contract_addr, initial_tokens))?;
    }

    Ok((contract_addr, asset_addr))
}

// ============================================================================
// Integration Tests: ERC-20 Token + Metadata Extension + ERC-20 Wrapper
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
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let name = contract.name().call().await?.name;
        assert_eq!(name, WRAPPED_TOKEN_NAME.to_owned());

        let symbol = contract.symbol().call().await?.symbol;
        assert_eq!(symbol, WRAPPED_TOKEN_SYMBOL.to_owned());

        let decimals = contract.decimals().call().await?.decimals;
        assert_eq!(decimals, 18);

        Ok(())
    }
}

mod deposit_to {

    use super::*;

    #[e2e::test]
    async fn executes_with_approval(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let alice_address = alice.address();
        let asset = ERC20Mock::new(asset_addr, &alice.wallet);
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

        //     _ = watch!(asset.mint(alice.address(), uint!(1000_U256)))?;

        _ = asset.approve(alice_address, uint!(1000_U256));

        _ = watch!(contract.depositFor(alice_address, uint!(1000_U256)))?;
        Ok(())
    }

    #[e2e::test]
    async fn reverts_for_invalid_sender(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .depositFor(contract_addr, uint!(1000_U256))
            .call()
            .await
            .expect_err("should return `InvalidSender`");
        assert!(err.reverted_with(Erc20Wrapper::ERC20InvalidSender {
            sender: contract_addr
        }));
        Ok(())
    }

    #[e2e::test]
    async fn reverts_minting_to_wrapper_contract(alice: Account) -> Result<()> {
        let alice_addr: Address = alice.address();
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .depositFor(alice.address(), uint!(1000_U256))
            .call()
            .await
            .expect_err("should return `InvalidReceiver`");
        assert!(err.reverted_with(Erc20Wrapper::ERC20InvalidSender {
            sender: alice_addr
        }));
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_missing_approval(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let alice_address = alice.address();
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .depositFor(alice_address, uint!(1000_U256))
            .call()
            .await
            .expect_err("should return `SafeErc20FailedOperation`");
        assert!(err.reverted_with(Erc20Wrapper::SafeErc20FailedOperation {
            token: asset_addr
        }));
        Ok(())
    }

    #[e2e::test]
    async fn reverts_when_inssuficient_balance(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let alice_address = alice.address();
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .depositFor(alice_address, uint!(1000_U256))
            .call()
            .await
            .expect_err("should return `ERC20InsufficientBalance`");
        assert!(err.reverted_with(Erc20Wrapper::ERC20InsufficientBalance {
            sender: alice_address,
            balance: uint!(0_U256),
            needed: uint!(1000_U256)
        }));
        Ok(())
    }

    #[e2e::test]
    async fn reflects_balance_after_deposit_for(alice: Account) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let alice_address = alice.address();
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        Ok(())
    }
}

mod withdraw_to {
    use super::*;

    #[e2e::test]
    async fn success(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        Ok(())
    }

    #[e2e::test]
    async fn reverts_for_invalid_sender(alice: Account) -> Result<()> {
        let (contract_addr, _) = deploy(&alice, U256::ZERO).await?;
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        let err = contract
            .withdrawTo(contract_addr, uint!(1000_U256))
            .call()
            .await
            .expect_err("should return `InvalidReciver`");
        assert!(err.reverted_with(Erc20Wrapper::ERC20InvalidReceiver {
            receiver: contract_addr
        }));
        Ok(())
    }

    #[e2e::test]
    async fn reflects_balance_after_withdraw_to(
        alice: Account,
        bob: Account,
    ) -> Result<()> {
        let (contract_addr, asset_addr) = deploy(&alice, U256::ZERO).await?;
        let alice_address = alice.address();
        let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
        Ok(())
    }
}
