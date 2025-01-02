#![cfg(feature = "e2e")]

use abi::Erc4626;
use alloy::{
    primitives::{b256, keccak256, Address, B256, U256,uint},
    sol,
    sol_types::SolType,
};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;
use stylus_sdk::contract::address;

use crate::Erc4626Example::constructorCall;

mod abi;

const ADDRESS : Address = Address::ZERO;

sol!("src/constructor.sol");

impl Default for constructorCall {
    fn default() -> Self {
        ctr(ADDRESS)
    }
}

fn ctr(asset: Address) -> constructorCall {
    constructorCall {
        asset_:asset
    }
}


#[e2e::test]
async fn error_when_exceeded_max_deposit(
    alice: Account,
    bob: Account,
) -> Result<()> {
     let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    // let balance = uint!(10_U256);
    // let value = uint!(11_U256);

    // let _ = watch!(contract_alice.mint(alice.address(), balance))?;

    // let Erc20::balanceOfReturn { balance: initial_alice_balance } =
    //     contract_alice.balanceOf(alice_addr).call().await?;
    // let Erc20::balanceOfReturn { balance: initial_bob_balance } =
    //     contract_alice.balanceOf(bob_addr).call().await?;
    // let Erc20::totalSupplyReturn { totalSupply: initial_supply } =
    //     contract_alice.totalSupply().call().await?;

    // let err = send!(contract_alice.transfer(bob_addr, value))
    //     .expect_err("should not transfer when insufficient balance");
    // assert!(err.reverted_with(Erc20::ERC20InsufficientBalance {
    //     sender: alice_addr,
    //     balance,
    //     needed: value
    // }));

    // let Erc20::balanceOfReturn { balance: alice_balance } =
    //     contract_alice.balanceOf(alice_addr).call().await?;
    // let Erc20::balanceOfReturn { balance: bob_balance } =
    //     contract_alice.balanceOf(bob_addr).call().await?;
    // let Erc20::totalSupplyReturn { totalSupply: supply } =
    //     contract_alice.totalSupply().call().await?;

    // assert_eq!(initial_alice_balance, alice_balance);
    // assert_eq!(initial_bob_balance, bob_balance);
    // assert_eq!(initial_supply, supply);

    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_mint(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_withdraw(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_redeem(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    Ok(())
}
