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

sol!("src/constructor.sol");

fn ctr(
    asset_: Address,
    name: String,
    symbol: String,
) -> constructorCall {
    constructorCall {
        name_: name,
        symbol_: symbol,
        asset_: asset_,
    }
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
    let decimals = contract.decimals().call().await?.decimals;
    let asset = contract.asset().call().await?.asset;
    assert_eq!(name, VALUT_NAME.to_owned());
    assert_eq!(symbol, VALUT_SYMBOL.to_owned());
    assert_eq!(decimals, 18);
    assert_eq!(asset, mock_token_address);
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
    let contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_mint(
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
    let contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_withdraw(
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
    let contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}

#[e2e::test]
async fn error_when_exceeded_max_redeem(
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
    let contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}
