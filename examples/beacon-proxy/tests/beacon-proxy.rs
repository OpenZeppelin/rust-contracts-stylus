#![cfg(feature = "e2e")]

use abi::BeaconProxyExample;
use alloy::{
    primitives::{uint, U256},
    sol_types::SolCall,
};
use e2e::{constructor, receipt, send, watch, Account, EventExt, Revert};
use eyre::Result;
use mock::{erc20, erc20::ERC20Mock};
use stylus_sdk::abi::Bytes;

mod abi;
mod mock;

fn zero_bytes() -> Bytes {
    vec![].into()
}

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let beacon_addr = alice
        .as_deployer()
        .with_constructor(constructor!(implementation_addr, alice.address()))
        .with_example_name("upgradeable-beacon")
        .deploy()
        .await?
        .contract_address;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(beacon_addr, zero_bytes()))
        .deploy()
        .await?
        .contract_address;
    let contract = BeaconProxyExample::new(contract_addr, &alice.wallet);

    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, implementation_addr);

    let beacon = contract.getBeacon().call().await?.beacon;
    assert_eq!(beacon, beacon_addr);

    Ok(())
}

#[e2e::test]
async fn constructs_with_data(alice: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let beacon_addr = alice
        .as_deployer()
        .with_constructor(constructor!(implementation_addr, alice.address()))
        .with_example_name("upgradeable-beacon")
        .deploy()
        .await?
        .contract_address;

    // mint 1000 tokens.
    let amount = uint!(1000_U256);

    let data = ERC20Mock::mintCall { account: alice.address(), value: amount };
    let data: Bytes = data.abi_encode().into();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(beacon_addr, data.clone()))
        .deploy()
        .await?
        .contract_address;
    let contract = BeaconProxyExample::new(contract_addr, &alice.wallet);

    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, implementation_addr);

    let beacon = contract.getBeacon().call().await?.beacon;
    assert_eq!(beacon, beacon_addr);

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
    let beacon_addr = alice
        .as_deployer()
        .with_constructor(constructor!(implementation_addr, alice.address()))
        .with_example_name("upgradeable-beacon")
        .deploy()
        .await?
        .contract_address;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(beacon_addr, zero_bytes()))
        .deploy()
        .await?
        .contract_address;
    let contract = BeaconProxyExample::new(contract_addr, &alice.wallet);

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

    assert!(receipt.emits(BeaconProxyExample::Transfer {
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
    let beacon_addr = alice
        .as_deployer()
        .with_constructor(constructor!(implementation_addr, alice.address()))
        .with_example_name("upgradeable-beacon")
        .deploy()
        .await?
        .contract_address;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor!(beacon_addr, zero_bytes()))
        .deploy()
        .await?
        .contract_address;
    let contract = BeaconProxyExample::new(contract_addr, &alice.wallet);

    let invalid_amount = uint!(1000_U256);

    let err = send!(contract.transfer(bob.address(), invalid_amount))
        .expect_err("should revert");
    assert!(err.reverted_with(BeaconProxyExample::ERC20InsufficientBalance {
        sender: alice.address(),
        balance: U256::ZERO,
        needed: invalid_amount,
    }),);

    Ok(())
}
