#![cfg(feature = "e2e")]

use abi::Erc721;

mod abi;

sol!("src/constructor.sol");

// ============================================================================
// Integration Tests: ERC-721 Wrapper Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {}
