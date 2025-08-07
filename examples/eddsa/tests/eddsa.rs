#![cfg(feature = "e2e")]

use alloy_primitives::{hex, U256};
use e2e::Account;
use eyre::Result;

use crate::abi::EddsaExample;

mod abi;

// ============================================================================
// Integration Tests: EDDSA
// ============================================================================

#[e2e::test]
async fn eddsa_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = EddsaExample::new(contract_addr, &alice.wallet);

    let secret_key = hex!(
        "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb"
    );
    let msg = hex!("72");

    let EddsaExample::signReturn { signature } = contract
        .sign(U256::from_le_bytes(secret_key), msg.into())
        .call()
        .await?;

    let expected_signature = hex!("92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00");

    assert_eq!(&signature.to_vec(), &expected_signature);

    Ok(())
}
