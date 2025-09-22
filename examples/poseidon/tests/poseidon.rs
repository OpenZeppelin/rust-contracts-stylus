#![cfg(feature = "e2e")]

use alloy_primitives::{hex, uint, U256};
use e2e::Account;
use eyre::Result;

use crate::abi::PoseidonExample;

mod abi;

// ============================================================================
// Integration Tests: Poseidon
// ============================================================================

#[e2e::test]
async fn poseidon_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PoseidonExample::new(contract_addr, &alice.wallet);

    let PoseidonExample::hashReturn { hash } =
        contract.hash([uint!(123_U256), uint!(123456_U256)]).call().await?;

    let expected = U256::from_be_slice(&hex!(
        "16f70722695a5829a59319fbf746df957a513fdf72b070a67bb72db08070e5de"
    ));

    assert_eq!(hash, expected);

    Ok(())
}
