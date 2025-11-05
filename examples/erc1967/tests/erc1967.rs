#![cfg(feature = "e2e")]

use abi::Erc1967Example;
use alloy::{
    primitives::{uint, Address, U256},
    sol_types::SolCall,
};
use e2e::{
    constructor, receipt, send, watch, Account, Constructor, EventExt, Revert,
};
use eyre::Result;
use mock::{erc20, erc20::ERC20Mock};
use stylus_sdk::abi::Bytes;

mod abi;
mod mock;

fn ctr(implementation: Address, data: &Bytes) -> Constructor {
    constructor!(implementation, data.clone())
}

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(implementation_addr, &vec![].into()))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc1967Example::new(contract_addr, &alice.wallet);

    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, implementation_addr);

    Ok(())
}

#[e2e::test]
async fn constructs_with_data(alice: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;

    // mint 1000 tokens.
    let amount = uint!(1000_U256);

    let data = ERC20Mock::mintCall { account: alice.address(), value: amount };
    let data = data.abi_encode();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(implementation_addr, &data.into()))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc1967Example::new(contract_addr, &alice.wallet);

    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, implementation_addr);

    // check that the balance can be accurately fetched through the proxy.
    let balance = contract.balanceOf(alice.address()).call().await?.balance;
    assert_eq!(balance, amount);

    let total_supply = contract.totalSupply().call().await?.totalSupply;
    assert_eq!(total_supply, amount);

    Ok(())
}

#[e2e::test]
async fn fallback(alice: Account, bob: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(implementation_addr, &vec![].into()))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc1967Example::new(contract_addr, &alice.wallet);

    // verify initial balance is [`U256::ZERO`].
    let balance = contract.balanceOf(alice.address()).call().await?.balance;
    assert_eq!(balance, U256::ZERO);

    let total_supply = contract.totalSupply().call().await?.totalSupply;
    assert_eq!(total_supply, U256::ZERO);

    // mint 1000 tokens.
    let amount = uint!(1000_U256);
    watch!(contract.mint(alice.address(), amount))?;

    // check that the balance can be accurately fetched through the proxy.
    let balance = contract.balanceOf(alice.address()).call().await?.balance;
    assert_eq!(balance, amount);

    let total_supply = contract.totalSupply().call().await?.totalSupply;
    assert_eq!(total_supply, amount);

    // check that the balance can be transferred through the proxy.
    let receipt = receipt!(contract.transfer(bob.address(), amount))?;

    assert!(receipt.emits(Erc1967Example::Transfer {
        from: alice.address(),
        to: bob.address(),
        value: amount,
    }));

    let balance = contract.balanceOf(alice.address()).call().await?.balance;
    assert_eq!(balance, U256::ZERO);

    let balance = contract.balanceOf(bob.address()).call().await?.balance;
    assert_eq!(balance, amount);

    let total_supply = contract.totalSupply().call().await?.totalSupply;
    assert_eq!(total_supply, amount);

    Ok(())
}

#[e2e::test]
async fn fallback_returns_error(alice: Account, bob: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(implementation_addr, &vec![].into()))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc1967Example::new(contract_addr, &alice.wallet);

    let err = send!(contract.transfer(bob.address(), uint!(1000_U256)))
        .expect_err("should revert");
    assert!(err.reverted_with(Erc1967Example::ERC20InsufficientBalance {
        sender: alice.address(),
        balance: U256::ZERO,
        needed: uint!(1000_U256),
    }),);

    Ok(())
}
