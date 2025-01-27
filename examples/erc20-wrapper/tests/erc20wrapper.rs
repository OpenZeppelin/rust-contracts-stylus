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

fn ctr(asset: Address) -> constructorCall {
    Erc20WrapperExample::constructorCall {
        underlyingToken_: asset,
        name_: WRAPPED_TOKEN_NAME.to_owned(),
        symbol_: WRAPPED_TOKEN_SYMBOL.to_owned(),
    }
}

// ============================================================================
// Integration Tests: ERC-20 Token + Metadata Extension + ERC-20 Wrapper
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let asset_address = erc20::deploy(&alice.wallet).await?;
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(asset_address))
        .deploy()
        .await?
        .address()?;
    let contract = Erc20Wrapper::new(contract_addr, &alice.wallet);
    let name = contract.name().call().await?.name;
    let symbol = contract.symbol().call().await?.symbol;
    let decimals = contract.decimals().call().await?.decimals;

    assert_eq!(name, WRAPPED_TOKEN_NAME.to_owned());
    assert_eq!(symbol, WRAPPED_TOKEN_SYMBOL.to_owned());
    assert_eq!(decimals, 18);

    Ok(())
}
