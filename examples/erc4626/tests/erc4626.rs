#![cfg(feature = "e2e")]

use std::println;

use abi::Erc4626;
use alloy::{primitives::Address, sol};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;
use mock::{token, token::ERC20Mock};
use stylus_sdk::contract::address;

use crate::Erc4626Example::constructorCall;

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";

const VALUT_NAME: &str = "Test Token Valut";
const VALUT_SYMBOL: &str = "TST Valut";

mod abi;
mod mock;

const ADDRESS: Address = Address::ZERO;

sol!("src/constructor.sol");

fn ctr(asset: Address, name: String, symbol: String) -> constructorCall {
    constructorCall { asset_: asset, name_: name, symbol_: symbol }
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let mock_token_address =
        token::deploy(&alice.wallet, TOKEN_NAME, TOKEN_SYMBOL).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(
            mock_token_address,
            VALUT_NAME.to_string(),
            VALUT_SYMBOL.to_string(),
        ))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);
    let name = contract.name().call().await?.name;
    let symbol = contract.symbol().call().await?.symbol;
    let asset = contract.asset().call().await?.asset;
    println!("name: {}, symbol: {} asset: {}", name, symbol, asset);
    // assert_eq!(name, VALUT_NAME.to_owned());
    // assert_eq!(symbol, VALUT_SYMBOL.to_owned());
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_deposit(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let mock_token_address =
        token::deploy(&alice.wallet, TOKEN_NAME, TOKEN_SYMBOL).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(
            mock_token_address,
            VALUT_NAME.to_string(),
            VALUT_SYMBOL.to_string(),
        ))
        .deploy()
        .await?
        .address()?;
    let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
    // let alice_addr = alice.address();
    // let bob_addr = bob.address();

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

// #[e2e::test]
// async fn error_when_exceeded_max_mint(
//     alice: Account,
//     bob: Account,
// ) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
//     let alice_addr = alice.address();
//     let bob_addr = bob.address();

//     Ok(())
// }

// #[e2e::test]
// async fn error_when_exceeded_max_withdraw(
//     alice: Account,
//     bob: Account,
// ) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
//     let alice_addr = alice.address();
//     let bob_addr = bob.address();

//     Ok(())
// }

// #[e2e::test]
// async fn error_when_exceeded_max_redeem(
//     alice: Account,
//     bob: Account,
// ) -> Result<()> {
//     let contract_addr = alice
//         .as_deployer()
//         .with_default_constructor::<constructorCall>()
//         .deploy()
//         .await?
//         .address()?;
//     let contract_alice = Erc4626::new(contract_addr, &alice.wallet);
//     let alice_addr = alice.address();
//     let bob_addr = bob.address();

//     Ok(())
// }
