#![cfg(feature = "e2e")]

use abi::PrecompilesExample;
use alloy::primitives::{address, b256, uint, Address, B256};
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

    let a = B1024::from_slice(&[0x00; 128]);
    let b = B1024::from_slice(&[0x01; 128]);
    let result =
        contract.callBls12G1Add(a.into(), b.into()).call().await?.result;

    assert_eq!(B1024::from_slice(&result), B1024::from_slice(&[0x01; 128]));

    Ok(())
}
