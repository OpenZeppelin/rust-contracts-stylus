#![cfg(feature = "e2e")]

use e2e::Account;
use eyre::Result;
use openzeppelin_crypto::arithmetic::uint::{from_str_hex, U256};

use crate::abi::PedersenExample;

mod abi;

// ============================================================================
// Integration Tests: Pedersen
// ============================================================================

#[e2e::test]
async fn pedersen_works(alice: Account) -> Result<()> {
    let input_1: U256 = from_str_hex::<4>(
        "3d937c035c878245caf64531a5756109c53068da139362728feb561405371cb",
    );
    let input_2: U256 = from_str_hex::<4>(
        "208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a",
    );
    let expected = from_str_hex::<4>(
        "30e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662",
    );

    let input_1: alloy_primitives::U256 = input_1.into();
    let input_2: alloy_primitives::U256 = input_2.into();
    let expected: alloy_primitives::U256 = expected.into();

    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PedersenExample::new(contract_addr, &alice.wallet);

    let PedersenExample::hashReturn { hash } =
        contract.hash([input_1, input_2]).call().await?;

    assert_eq!(hash, expected);

    Ok(())
}
