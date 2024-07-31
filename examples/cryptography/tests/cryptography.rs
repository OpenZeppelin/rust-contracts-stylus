#![cfg(feature = "e2e")]

use abi::Crypto;
use alloy::{
    primitives::{address, b256, uint, Address, B256},
    sol,
    sol_types::SolConstructor,
};
use e2e::{Account, Revert};

mod abi;

sol!("src/constructor.sol");

async fn deploy(account: &Account) -> eyre::Result<Address> {
    let args = CryptoExample::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(account.url(), &account.pk(), Some(args)).await
}

// ============================================================================
// Integration Tests: EIP-712
// ============================================================================
