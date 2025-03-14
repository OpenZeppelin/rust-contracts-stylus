#![cfg(feature = "e2e")]

use abi::Erc721Wrapper;
use alloy::{primitives::Address, sol};
use openzeppelin_stylus::token::erc721::Erc721;

mod abi;

sol!("src/constructor.sol");

fn ctr(asset_addr: Address) -> constructorCall {
    Erc721WrapperExample::constructorCall { underlyingToken_: asset_addr }
}

// ============================================================================
// Integration Tests: ERC-721 Wrapper Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    Ok(())
}
