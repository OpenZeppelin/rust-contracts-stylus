#![cfg(feature = "e2e")]

use std::assert_eq;

use abi::Erc1155Supply;
use alloy::{
    primitives::{uint, Address, U256},
    signers::k256::sha2::digest::typenum::uint,
};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};

mod abi;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(|_| U256::from(rand::random::<u32>())).collect()
}

fn random_values(size: usize) -> Vec<U256> {
    (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let Erc1155Supply::totalSupplyAllReturn { total_supply_all } =
        contract.totalSupplyAll().call().await?;

    assert_eq!(total_supply_all, uint!(0_U256));

    Ok(())
}

#[e2e::test]
async fn supply_of_zero_supply(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let token_ids = random_token_ids(1);
    let total_supply =
        contract.totalSupply(token_ids[0]).call().await?.total_supply;
    let total_supply_all =
        contract.totalSupplyAll().call().await?.total_supply_all;
    let exists = contract.exists(token_ids[0]).call().await?.existed;

    assert_eq!(total_supply, uint!(0_U256));
    assert_eq!(total_supply_all, uint!(0_U256));
    assert!(!exists);

    Ok(())
}

#[e2e::test]
async fn supply_with_zero_address_sender(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let token_ids = random_token_ids(1);
    let values = random_values(1);
    let _ = watch!(contract.mint(
        alice.address(),
        token_ids.clone(),
        values.clone()
    ));

    let total_supply =
        contract.totalSupply(token_ids[0]).call().await?.total_supply;
    let total_supply_all =
        contract.totalSupplyAll().call().await?.total_supply_all;
    let exists = contract.exists(token_ids[0]).call().await?.existed;

    assert_eq!(total_supply, values[0]);
    assert_eq!(total_supply_all, values[0]);
    assert!(exists);

    Ok(())
}

#[e2e::test]
async fn supply_batch(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let token_ids = random_token_ids(4);
    let values = random_values(4);
    let _ = watch!(contract.mint(
        alice.address(),
        token_ids.clone(),
        values.clone()
    ));

    for i in 0..4 {
        let total_supply =
            contract.totalSupply(token_ids[i]).call().await?.total_supply;
        assert_eq!(total_supply, values[i]);
    }

    for i in 0..4 {
        let exists = contract.exists(token_ids[i]).call().await?.existed;
        assert!(exists);
    }

    let total_supply_all =
        contract.totalSupplyAll().call().await?.total_supply_all;

    assert_eq!(total_supply_all, values.iter().sum());

    Ok(())
}
