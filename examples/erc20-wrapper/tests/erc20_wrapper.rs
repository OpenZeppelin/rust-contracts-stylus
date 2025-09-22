#![cfg(feature = "e2e")]

use abi::{Erc20, Erc20Wrapper};
use alloy::primitives::{uint, Address, U256};
use e2e::{constructor, receipt, watch, Account, Constructor, EventExt};
use eyre::Result;

mod abi;
mod mock;

use mock::{erc20, erc20::ERC20Mock};

const DECIMALS: u8 = 18;

fn ctr(asset_addr: Address) -> Constructor {
    constructor!(asset_addr)
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
        .contract_address;

    if initial_tokens > U256::ZERO {
        let asset = ERC20Mock::new(asset_addr, &account.wallet);
        watch!(asset.mint(account.address(), initial_tokens))?;
    }

    Ok((contract_addr, asset_addr))
}

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc20Wrapper::new(contract_addr, alice.wallet);

    let underlying = contract.underlying().call().await?.underlying;
    assert_eq!(underlying, asset_address);

    let decimals = contract.decimals().call().await?.decimals;
    assert_eq!(decimals, DECIMALS);

    Ok(())
}

#[e2e::test]
async fn deposit_for_success(alice: Account) -> Result<()> {
    let initial_supply = uint!(1000_U256);
    let (contract_addr, asset_addr) = deploy(&alice, initial_supply).await?;
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

#[e2e::test]
async fn withdraw_to_success(alice: Account) -> Result<()> {
    let initial_tokens = uint!(1000_U256);
    let (contract_addr, asset_addr) = deploy(&alice, initial_tokens).await?;

    let asset = ERC20Mock::new(asset_addr, &alice.wallet);
    let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);

    watch!(asset.approve(contract_addr, initial_tokens))?;

    watch!(contract.depositFor(alice.address(), initial_tokens))?;

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
