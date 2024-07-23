#![cfg(feature = "e2e")]

use abi::Crypto;
use alloy::{
    primitives::{eip191_hash_message, fixed_bytes, Address, FixedBytes, B256},
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

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

const MESSAGE: FixedBytes<4> = fixed_bytes!("deadbeef");

#[e2e::test]
async fn recovers_from_v_r_s(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = Crypto::new(contract_addr, &alice.wallet);

    let hash = hash(&*MESSAGE);
    let signature = alice.sign_hash(&hash).await;

    let recovered =
        signature.recover_address_from_msg(MESSAGE).expect("should recover");
    assert_eq!(recovered, alice.address());

    let Crypto::recover_2Return { recovered } = contract
        .recover_2(
            hash,
            signature.v().to_u64() as u8,
            signature.r().into(),
            signature.s().into(),
        )
        .call()
        .await?;

    assert_eq!(alice.address(), recovered);

    Ok(())
}

#[e2e::test]
async fn recovers_from_signature(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = Crypto::new(contract_addr, &alice.wallet);

    let hash = hash(&*MESSAGE);
    let signature = alice.sign_hash(&hash).await;

    let recovered =
        signature.recover_address_from_msg(MESSAGE).expect("should recover");
    assert_eq!(recovered, alice.address());

    let Crypto::recover_0Return { recovered } =
        contract.recover_0(hash, signature.as_bytes().into()).call().await?;

    assert_eq!(alice.address(), recovered);

    Ok(())
}
