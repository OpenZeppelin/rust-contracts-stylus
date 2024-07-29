#![cfg(feature = "e2e")]

use abi::Crypto;
use alloy::{
    primitives::{
        address, b256, bytes, eip191_hash_message, fixed_bytes, Address, Bytes,
        FixedBytes, B256, U256,
    },
    sol,
    sol_types::SolConstructor,
};
use e2e::{Account, Revert};
use eyre::Result;

mod abi;

sol!("src/constructor.sol");

async fn deploy(account: &Account) -> eyre::Result<Address> {
    let args = CryptoExample::constructorCall {};
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(account.url(), &account.pk(), Some(args)).await
}

fn hash(message: &[u8]) -> B256 {
    eip191_hash_message(message)
}

const EXAMPLE_HASH: B256 =
    b256!("a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2");

const EXAMPLE_ADDRESS: Address =
    address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

const TEST_MESSAGE: Bytes =
    bytes!("7dbaf558b0a1a5dc7a67202117ab143c1d8605a983e4a743bc06fcc03162dc0d");
const WRONG_MESSAGE: Bytes =
    bytes!("2d0828dd7c97cff316356da3c16c68ba2316886a0e05ebafb8291939310d51a3");
const NON_HASH_MESSAGE: Bytes = bytes!("abcd");

const SIG_WITHOUT_V27: FixedBytes<64> = fixed_bytes!("5d99b6f7f6d1f73d1a26497f2b1c89b24c0993913f86e9a2d02cd69887d9c94f3c880358579d811b21dd1b7fd9bb01c1d81d10e69f0384e675c32b39643be892");
const V27: FixedBytes<1> = fixed_bytes!("1b");
const SIG_V27_SIGNER: Address =
    address!("2cc1166f6212628A0deEf2B33BEFB2187D35b86c");

const SIG_WITHOUT_V28: FixedBytes<64> = fixed_bytes!("331fe75a821c982f9127538858900d87d3ec1f9f737338ad67cad133fa48feff48e6fa0c18abc62e42820f05943e47af3e9fbe306ce74d64094bdf1691ee53e0");
const V28: FixedBytes<1> = fixed_bytes!("1c");
const SIG_V28_SIGNER: Address =
    address!("1E318623aB09Fe6de3C9b8672098464Aeda9100E");

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = Crypto::new(contract_addr, &alice.wallet);

    let v = 28;
    let r = b256!(
        "65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee82"
    );
    let s = b256!(
        "3eb5a6982b540f185703492dab77b863a88ce01f27e21ade8b2879c10fc9e653"
    );
    let Crypto::recoverReturn { recovered } =
        contract.recover(EXAMPLE_HASH, v, r, s).call().await?;

    assert_eq!(EXAMPLE_ADDRESS, recovered);

    Ok(())
}

#[e2e::test]
async fn recovers_from_v_r_s(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = Crypto::new(contract_addr, &alice.wallet);

    let hash = hash(&TEST_MESSAGE);
    let signature = alice.sign_hash(&hash).await;

    let Crypto::recoverReturn { recovered } = contract
        .recover(
            hash,
            signature
                .v()
                .y_parity_byte_non_eip155()
                .expect("should be non-EIP155 signature"),
            signature.r().into(),
            signature.s().into(),
        )
        .call()
        .await?;

    assert_eq!(alice.address(), recovered);

    Ok(())
}
