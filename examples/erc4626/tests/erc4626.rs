#![cfg(feature = "e2e")]

use std::println;
use alloy_primitives::U256;

use abi::Erc4626;
use alloy::{contract,  primitives::{uint, Address}, sol};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;
use mock::{token, token::ERC20Mock,token::MockErc20Abi};
use stylus_sdk::contract::address;

use crate::Erc4626Example::constructorCall;

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "TTK";

const VALUT_NAME: &str = "Test Token Valut";
const VALUT_SYMBOL: &str = "TST Valut";

mod abi;
mod mock;

sol!("src/constructor.sol");

fn ctr(asset: Address) -> constructorCall {
    constructorCall { name_: VALUT_NAME.to_owned(), symbol_: VALUT_SYMBOL.to_owned(), asset_: asset }
}

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let mock_token_address =
        token::deploy(&alice.wallet, TOKEN_NAME, TOKEN_SYMBOL).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(mock_token_address))
        .deploy()
        .await?
        .address()?;
    let token_contract = MockErc20Abi::new(mock_token_address, &alice.wallet);
    let name = token_contract.name().call().await?.name;
    let symbol = token_contract.symbol().call().await?.symbol;
    assert_eq!(name, TOKEN_NAME.to_owned());
    assert_eq!(symbol, TOKEN_SYMBOL.to_owned());

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
async fn deposit(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let mock_token_address =
        token::deploy(&alice.wallet, TOKEN_NAME, TOKEN_SYMBOL).await?;
     let vault_addr = alice
        .as_deployer()
        .with_constructor(ctr(mock_token_address))
        .deploy()
        .await?
        .address()?;
    let asset = MockErc20Abi::new(mock_token_address, &alice.wallet);
    let vault = Erc4626::new(vault_addr, &alice.wallet);

    // Mint token
    let mint_receipt = asset.mint(alice.address(), uint!(100_U256)).send().await?;
    println!("{:?}", mint_receipt);

    let _ = asset.approve(alice.address(), U256::MAX).send().await?;
    let _ = vault.approve(alice.address(), U256::MAX).send().await?;

    let max_mint = vault.maxMint(bob.address()).call().await?._0;
    let preview_deposit = vault.previewDeposit(uint!(1_U256)).call().await?._0;
    let deposit = vault.deposit(uint!(1_U256), bob.address()).call().await?._0;
    assert_eq!(max_mint, U256::MAX);
    assert_eq!(preview_deposit, uint!(1_U256));

    let asset_balance = asset.balanceOf(bob.address()).call().await?.balance;
    //assert_eq!(asset_balance, uint!(1_U256));

    // let valut_balance = contract.balanceOf(bob.address()).call().await?.balance;
    // assert_eq!(valut_balance, uint!(1_U256));

    Ok(())
}


#[e2e::test]
async fn mint(
    alice: Account,
    bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn withdraw(
    alice: Account,
    bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn redeem(
    alice: Account,
    bob: Account,
) -> Result<()> {
    Ok(())
}


#[e2e::test]
async fn deposit_inflation_attack(
    alice: Account,
    bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn mint_inflation_attack(
    alice: Account,
    bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn withdraw_inflation_attack(
    alice: Account,
    bob: Account,
) -> Result<()> {
    Ok(())
}

#[e2e::test]
async fn redeem_inflation_attack(
    alice: Account,
    bob: Account,
) -> Result<()> {
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
        .with_constructor(ctr(mock_token_address))
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
        .with_constructor(ctr(mock_token_address))
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
        .with_constructor(ctr(mock_token_address))
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
        .with_constructor(ctr(mock_token_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc4626::new(contract_addr, &alice.wallet);
    Ok(())
}
