#![cfg(feature = "e2e")]

use abi::PrecompilesExample;
use alloy::{
    hex::FromHex,
    primitives::{address, b256, uint, Address, Bytes, B256},
};
use alloy_primitives::aliases::B1024;
use e2e::{Account, Revert};
use eyre::Result;
use openzeppelin_stylus::utils::cryptography::ecdsa::SIGNATURE_S_UPPER_BOUND;

mod abi;

const HASH: B256 =
    b256!("a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2");

const V: u8 = 28;
const R: B256 =
    b256!("65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee82");
const S: B256 =
    b256!("3eb5a6982b540f185703492dab77b863a88ce01f27e21ade8b2879c10fc9e653");

const ADDRESS: Address = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let PrecompilesExample::recoverReturn { recovered } =
        contract.recover(HASH, V, R, S).call().await?;

    assert_eq!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn different_hash_recovers_different_address(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let hash = b256!(
        "65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee82"
    );
    let PrecompilesExample::recoverReturn { recovered } =
        contract.recover(hash, V, R, S).call().await?;

    assert_ne!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn different_v_recovers_different_address(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let v = 27;

    let PrecompilesExample::recoverReturn { recovered } =
        contract.recover(HASH, v, R, S).call().await?;

    assert_ne!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn different_r_recovers_different_address(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let r = b256!(
        "b814eaab5953337fed2cf504a5b887cddd65a54b7429d7b191ff1331ca0726b1"
    );

    let PrecompilesExample::recoverReturn { recovered } =
        contract.recover(HASH, V, r, S).call().await?;

    assert_ne!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn different_s_recovers_different_address(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let s = b256!(
        "3eb5a6982b540f185703492dab77b863a99ce01f27e21ade8b2879c10fc9e653"
    );
    let PrecompilesExample::recoverReturn { recovered } =
        contract.recover(HASH, V, R, s).call().await?;

    assert_ne!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn recovers_from_v_r_s(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let signature = alice.sign_hash(&HASH).await;

    // converted to non-eip155 `v` value
    // see https://eips.ethereum.org/EIPS/eip-155
    let v_byte = signature.v() as u8 + 27;

    let PrecompilesExample::recoverReturn { recovered } = contract
        .recover(HASH, v_byte, signature.r().into(), signature.s().into())
        .call()
        .await?;

    assert_eq!(alice.address(), recovered);

    Ok(())
}

#[e2e::test]
async fn rejects_v0_with_invalid_signature_error(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let wrong_v = 0;
    let err = contract
        .recover(HASH, wrong_v, R, S)
        .call()
        .await
        .expect_err("should return `ECDSAInvalidSignature`");

    assert!(err.reverted_with(PrecompilesExample::ECDSAInvalidSignature {}));

    Ok(())
}

#[e2e::test]
async fn rejects_v1_with_invalid_signature_error(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let wrong_v = 0;
    let err = contract
        .recover(HASH, wrong_v, R, S)
        .call()
        .await
        .expect_err("should return `ECDSAInvalidSignature`");

    assert!(err.reverted_with(PrecompilesExample::ECDSAInvalidSignature {}));

    Ok(())
}

#[e2e::test]
async fn error_when_higher_s(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let higher_s = SIGNATURE_S_UPPER_BOUND + uint!(1_U256);

    let higher_s = B256::from_slice(&higher_s.to_be_bytes_vec());

    let err = contract
        .recover(HASH, V, R, higher_s)
        .call()
        .await
        .expect_err("should return `ECDSAInvalidSignature`");

    assert!(err.reverted_with(PrecompilesExample::ECDSAInvalidSignatureS {
        s: higher_s
    }));

    Ok(())
}

// ============================================================================
// Integration Tests: BLS12-G1ADD
// ============================================================================

#[e2e::test]
async fn bls12_g1_add_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let a = B1024::from_hex("000000000000000000000000000000000572cbea904d67468808c8eb50a9450c9721db309128012543902d0ac358a62ae28f75bb8f1c7c42c39a8c5529bf0f4e00000000000000000000000000000000166a9d8cabc673a322fda673779d8e3822ba3ecb8670e461f73bb9021d5fd76a4c56d9d4cd16bd1bba86881979749d28").expect("should be valid hex for 'a'");
    let b = B1024::from_hex("0000000000000000000000000000000009ece308f9d1f0131765212deca99697b112d61f9be9a5f1f3780a51335b3ff981747a0b2ca2179b96d2c0c9024e522400000000000000000000000000000000032b80d3a6f5b09f8a84623389c5f80ca69a0cddabc3097f9d9c27310fd43be6e745256c634af45ca3473b0590ae30d1").expect("should be valid hex for 'b'");

    let result =
        contract.callBls12G1Add(a.into(), b.into()).call().await?.result;

    assert_eq!(result,
    Bytes::from_hex("0000000000000000000000000000000010e7791fb972fe014159aa33a98622da3cdc98ff707965e536d8636b5fcc5ac7a91a8c46e59a00dca575af0f18fb13dc0000000000000000000000000000000016ba437edcc6551e30c10512367494bfb6b01cc6681e8a4c3cd2501832ab5c4abc40b4578b85cbaffbf0bcd70d67c6e2").expect("should be valid hex for 'result'"));

    Ok(())
}
