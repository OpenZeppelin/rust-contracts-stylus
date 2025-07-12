#![cfg(feature = "e2e")]

use e2e::Account;
use eyre::Result;
use openzeppelin_crypto::arithmetic::{
    uint::{from_str_hex, U256},
    BigInteger,
};

use crate::abi::PedersenExample;

mod abi;

// ============================================================================
// Integration Tests: Pedersen
// ============================================================================

fn to_alloy_u256(value: &U256) -> alloy_primitives::U256 {
    alloy_primitives::U256::from_le_slice(&value.into_bytes_le())
}

#[e2e::test]
async fn hash_returns_expected_pedersen_result(alice: Account) -> Result<()> {
    let input_1 = to_alloy_u256(&from_str_hex(
        "3d937c035c878245caf64531a5756109c53068da139362728feb561405371cb",
    ));
    let input_2 = to_alloy_u256(&from_str_hex(
        "208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a",
    ));

    let expected = to_alloy_u256(&from_str_hex(
        "30e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662",
    ));

    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PedersenExample::new(contract_addr, &alice.wallet);

    let PedersenExample::hashReturn { hash } =
        contract.hash([input_1, input_2]).call().await?;

    assert_eq!(hash, expected);

    Ok(())
}
