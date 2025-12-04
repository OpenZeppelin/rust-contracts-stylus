#![cfg(feature = "e2e")]

use abi::PrecompilesExample;
use alloy::primitives::{address, b256, Address, B256, U256};
use e2e::{Account, Revert};
use eyre::Result;
use openzeppelin_stylus::utils::cryptography::ecdsa::SIGNATURE_S_UPPER_BOUND;

mod abi;

const ECDSA_HASH: B256 =
    b256!("a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2");

const ECDSA_V: u8 = 28;
const ECDSA_R: B256 =
    b256!("65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee82");
const ECDSA_S: B256 =
    b256!("3eb5a6982b540f185703492dab77b863a88ce01f27e21ade8b2879c10fc9e653");

const ADDRESS: Address = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let recovered = contract
        .ecRecoverExample(ECDSA_HASH, ECDSA_V, ECDSA_R, ECDSA_S)
        .call()
        .await?
        .recovered;

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
    let recovered = contract
        .ecRecoverExample(hash, ECDSA_V, ECDSA_R, ECDSA_S)
        .call()
        .await?
        .recovered;

    assert_ne!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn different_v_recovers_different_address(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let v = 27;

    let recovered = contract
        .ecRecoverExample(ECDSA_HASH, v, ECDSA_R, ECDSA_S)
        .call()
        .await?
        .recovered;

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

    let recovered = contract
        .ecRecoverExample(ECDSA_HASH, ECDSA_V, r, ECDSA_S)
        .call()
        .await?
        .recovered;

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
    let recovered = contract
        .ecRecoverExample(ECDSA_HASH, ECDSA_V, ECDSA_R, s)
        .call()
        .await?
        .recovered;

    assert_ne!(ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn recovers_from_v_r_s(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let signature = alice.sign_hash(&ECDSA_HASH).await;

    // converted to non-eip155 `v` value
    // see https://eips.ethereum.org/EIPS/eip-155
    let v_byte = u8::from(signature.v()) + 27;

    let recovered = contract
        .ecRecoverExample(
            ECDSA_HASH,
            v_byte,
            signature.r().into(),
            signature.s().into(),
        )
        .call()
        .await?
        .recovered;

    assert_eq!(alice.address(), recovered);

    Ok(())
}

#[e2e::test]
async fn rejects_v0_with_invalid_signature_error(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    let wrong_v = 0;
    let err = contract
        .ecRecoverExample(ECDSA_HASH, wrong_v, ECDSA_R, ECDSA_S)
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
        .ecRecoverExample(ECDSA_HASH, wrong_v, ECDSA_R, ECDSA_S)
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

    let higher_s = SIGNATURE_S_UPPER_BOUND + U256::ONE;

    let higher_s = B256::from_slice(&higher_s.to_be_bytes_vec());

    let err = contract
        .ecRecoverExample(ECDSA_HASH, ECDSA_V, ECDSA_R, higher_s)
        .call()
        .await
        .expect_err("should return `ECDSAInvalidSignature`");

    assert!(err.reverted_with(PrecompilesExample::ECDSAInvalidSignatureS {
        s: higher_s
    }));

    Ok(())
}

#[e2e::test]
async fn p256_verify_returns_true_on_successful_verification(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    // Values from https://github.com/OffchainLabs/go-ethereum/blob/1a03a778cf634c4ed82780a5e1fad5adcd01489e/core/vm/testdata/precompiles/p256Verify.json#L3
    let hash: B256 = b256!(
        "bb5a52f42f9c9261ed4361f59422a1e30036e7c32b270c8807a419feca605023"
    );
    let r: B256 = b256!(
        "2ba3a8be6b94d5ec80a6d9d1190a436effe50d85a1eee859b8cc6af9bd5c2e18"
    );
    let s: B256 = b256!(
        "4cd60b855d442f5b3c7b11eb6c4e0ae7525fe710fab9aa7c77a67f79e6fadd76"
    );
    let x: B256 = b256!(
        "2927b10512bae3eddcfe467828128bad2903269919f7086069c8c4df6c732838"
    );
    let y: B256 = b256!(
        "c7787964eaac00e5921fb1498a60f4606766b3d9685001558d1a974e7341513e"
    );

    let result =
        contract.p256VerifyExample(hash, r, s, x, y).call().await?.result;

    assert!(result);

    Ok(())
}

#[e2e::test]
async fn p256_verify_returns_false_on_failed_verification(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = PrecompilesExample::new(contract_addr, &alice.wallet);

    // Values from https://github.com/OffchainLabs/go-ethereum/blob/1a03a778cf634c4ed82780a5e1fad5adcd01489e/core/vm/testdata/precompiles/p256Verify.json#L10
    let hash: B256 = b256!(
        "bb5a52f42f9c9261ed4361f59422a1e30036e7c32b270c8807a419feca605023"
    );
    let invalid_r: B256 = b256!(
        "d45c5740946b2a147f59262ee6f5bc90bd01ed280528b62b3aed5fc93f06f739"
    );
    let invalid_s: B256 = b256!(
        "b329f479a2bbd0a5c384ee1493b1f5186a87139cac5df4087c134b49156847db"
    );
    let x: B256 = b256!(
        "2927b10512bae3eddcfe467828128bad2903269919f7086069c8c4df6c732838"
    );
    let y: B256 = b256!(
        "c7787964eaac00e5921fb1498a60f4606766b3d9685001558d1a974e7341513e"
    );

    let result = contract
        .p256VerifyExample(hash, invalid_r, invalid_s, x, y)
        .call()
        .await?
        .result;

    assert!(!result);

    Ok(())
}
