#![cfg(feature = "e2e")]

use abi::Erc1967Example;
use alloy::{
    primitives::{Address, U256},
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

fn ctr(implementation: Address, data: Bytes) -> Constructor {
    constructor!(implementation, data.clone())
}

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let beacon_addr = alice
        .as_deployer()
        .with_constructor(constructor!(implementation_addr, alice.address()))
        .deploy_from_example("upgradeable-beacon")
        .await?
        .contract_address;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(beacon_addr, vec![].into()))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc1967Example::new(contract_addr, &alice.wallet);

    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, implementation_addr);

    let beacon = contract.getBeacon().call().await?.beacon;
    assert_eq!(beacon, beacon_addr);

    Ok(())
}
