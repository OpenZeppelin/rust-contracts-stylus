#![cfg(feature = "e2e")]

use alloy_primitives::{bytes, hex, B256};
use e2e::{Account, ReceiptExt, Revert};
use eyre::Result;

use crate::abi::PoseidonExample;

mod abi;

// ============================================================================
// Integration Tests: Poseidon
// ============================================================================

#[e2e::test]
async fn poseidon_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = PoseidonExample::new(contract_addr, &alice.wallet);

    let PoseidonExample::hashReturn { hash } =
        contract.hash(bytes!("deadbeef")).call().await?;

    let expected = B256::from(hex!(
        "438ba31003629145d5a99d47392a014833076ab2fbd485ce446ce617cc83e03f"
    ));

    assert_eq!(hash, expected);

    Ok(())
}
