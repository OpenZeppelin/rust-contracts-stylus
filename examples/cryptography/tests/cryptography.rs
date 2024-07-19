#![cfg(feature = "e2e")]

use abi::Crypto;
use alloy::{
    primitives::{fixed_bytes, keccak256, Address, FixedBytes},
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

// ============================================================================
// Integration Tests: ECDSA
// ============================================================================

const MESSAGE: FixedBytes<4> = fixed_bytes!("deadbeef");

#[e2e::test]
async fn recovers(alice: Account) -> Result<()> {
    let contract_addr = deploy(&alice).await?;
    let contract = Crypto::new(contract_addr, &alice.wallet);

    let hash = keccak256(*MESSAGE);
    let signature = alice.sign_hash(&hash).await;

    let Crypto::recover_2Return { recovered } = contract
        .recover_2(
            hash,
            signature.v().y_parity_byte(),
            signature.r().into(),
            signature.s().into(),
        )
        .call()
        .await?;

    assert_eq!(alice.address(), recovered);

    Ok(())
}
