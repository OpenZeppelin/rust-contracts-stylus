#![cfg(feature = "e2e")]

use alloy_primitives::hex;
use e2e::Account;
use eyre::Result;
use openzeppelin_crypto::{
    curve::CurveGroup,
    eddsa::{Signature, SigningKey, VerifyingKey},
};

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
    let signing_key = SigningKey::from_bytes(&secret_key);

    // Verify with signed message.
    let message = b"Sign me!";
    let signature = signing_key.sign(message);
    let EddsaExample::verifyReturn { is_valid } = contract
        .verify(
            encode_verifying_key(signing_key.verifying_key()),
            encode_signature(signature),
            message.into(),
        )
        .call()
        .await?;
    assert!(is_valid);

    // Verify with a different message.
    let invalid_message = b"I'm not signed!";
    let EddsaExample::verifyReturn { is_valid } = contract
        .verify(
            encode_verifying_key(signing_key.verifying_key()),
            encode_signature(signature),
            invalid_message.into(),
        )
        .call()
        .await?;
    assert!(!is_valid);

    Ok(())
}

/// Non-canonical encoding of [`Signature`].
fn encode_signature(signature: Signature) -> [alloy_primitives::U256; 3] {
    let affine_r = signature.R.into_affine();
    [
        affine_r.x.into_bigint().into(),
        affine_r.y.into_bigint().into(),
        signature.s.into_bigint().into(),
    ]
}

/// Non-canonical encoding of [`VerifyingKey`].
fn encode_verifying_key(
    verifying_key: VerifyingKey,
) -> [alloy_primitives::U256; 2] {
    let affine_verifying_key = verifying_key.point.into_affine();
    [
        affine_verifying_key.x.into_bigint().into(),
        affine_verifying_key.y.into_bigint().into(),
    ]
}
