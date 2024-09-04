#![cfg(feature = "e2e")]

use abi::Erc1155;

mod abi;

// ============================================================================
// Integration Tests: ERC-1155 Token Standard
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {}
