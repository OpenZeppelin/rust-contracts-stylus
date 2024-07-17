#![cfg(feature = "e2e")]

use alloy::{primitives::Address, sol, sol_types::SolConstructor};
use e2e::Account;
use eyre::Result;

mod abi;

sol!("src/constructor.sol");

async fn deploy(account: &Account) -> eyre::Result<Address> {
    let args = CryptoExample::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(account.url(), &account.pk(), Some(args)).await
}

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    assert_ne!(contract_addr, Address::ZERO);
    Ok(())
}
