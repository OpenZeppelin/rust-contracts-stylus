#![cfg(feature = "e2e")]

use abi::Crypto;
use alloy::{
    primitives::{
        address, b256, eip191_hash_message, fixed_bytes, Address, FixedBytes,
        B256,
    },
    sol,
    sol_types::SolConstructor,
};
use e2e::Account;
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

const MESSAGE: FixedBytes<4> = fixed_bytes!("deadbeef");

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

#[e2e::test]
async fn ecrecover_works(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = Crypto::new(contract_addr, &alice.wallet);

    let hash = b256!(
        "a1de988600a42c4b4ab089b619297c17d53cffae5d5120d82d8a92d0bb3b78f2"
    );
    let v = 28;
    let r = b256!(
        "65e72b1cf8e189569963750e10ccb88fe89389daeeb8b735277d59cd6885ee82"
    );
    let s = b256!(
        "3eb5a6982b540f185703492dab77b863a88ce01f27e21ade8b2879c10fc9e653"
    );
    let address = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

    let Crypto::recover_2Return { recovered } =
        contract.recover_2(hash, v, r, s).call().await?;

    assert_eq!(address, recovered);

    Ok(())
}

// #[e2e::test]
// async fn recovers_from_v_r_s(alice: Account) -> Result<()> {
// let contract_addr = deploy(&alice).await?;
// let contract = Crypto::new(contract_addr, &alice.wallet);
//
// let hash = hash(&*MESSAGE);
// let signature = alice.sign_hash(&hash).await;
//
// let recovered =
// signature.recover_address_from_msg(MESSAGE).expect("should recover");
// assert_eq!(recovered, alice.address());
//
// let Crypto::recover_2Return { recovered } = contract
// .recover_2(
// hash,
// signature.v().to_u64() as u8,
// signature.r().into(),
// signature.s().into(),
// )
// .call()
// .await?;
//
// assert_eq!(alice.address(), recovered);
//
// Ok(())
// }
//
// #[e2e::test]
// async fn recovers_from_signature(alice: Account) -> Result<()> {
// let contract_addr = deploy(&alice).await?;
// let contract = Crypto::new(contract_addr, &alice.wallet);
//
// let hash = hash(&*MESSAGE);
// let signature = alice.sign_hash(&hash).await;
//
// let recovered =
// signature.recover_address_from_msg(MESSAGE).expect("should recover");
// assert_eq!(recovered, alice.address());
//
// let Crypto::recover_0Return { recovered } =
// contract.recover_0(hash, signature.as_bytes().into()).call().await?;
//
// assert_eq!(alice.address(), recovered);
//
// Ok(())
// }
