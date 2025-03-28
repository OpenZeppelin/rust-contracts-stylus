#![cfg(feature = "e2e")]

// use abi::Erc721Wrapper;
use alloy::{primitives::Address, sol};
use e2e::Account;
use eyre::Result;

mod abi;

use crate::Erc721WrapperExample::constructorCall;

sol!("src/constructor.sol");

fn ctr(asset_addr: Address) -> constructorCall {
    Erc721WrapperExample::constructorCall { underlyingToken_: asset_addr }
}

// ============================================================================
// Integration Tests: ERC-721 Wrapper Extension
// ============================================================================

#[e2e::test]
async fn constructs(_alice: Account) -> Result<()> {
    Ok(())
}
