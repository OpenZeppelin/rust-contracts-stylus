use ethers::prelude::*;
use eyre::Result;

use crate::infrastructure::{erc721::*, *};

#[tokio::test]
async fn mint() -> Result<()> {
    let E2EContext { alice, bob } = E2EContext::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = alice.mint(alice.wallet.address(), token_id).ctx_send().await?;
    let owner = alice.owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, alice.wallet.address());

    let balance = alice.balance_of(alice.wallet.address()).ctx_call().await?;
    assert!(balance >= U256::one());
    Ok(())
}

#[tokio::test]
async fn error_when_reusing_token_id() -> Result<()> {
    let E2EContext { alice, bob } = E2EContext::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = alice.mint(alice.wallet.address(), token_id).ctx_send().await?;
    let err = alice
        .mint(alice.wallet.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not mint a token id twice");
    err.assert(ERC721InvalidSender { sender: Address::zero() })
}

#[tokio::test]
async fn transfer() -> Result<()> {
    let E2EContext { alice, bob } = E2EContext::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = alice.mint(alice.wallet.address(), token_id).ctx_send().await?;
    let _ = alice
        .transfer_from(alice.wallet.address(), bob.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let owner = bob.owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, bob.wallet.address());
    Ok(())
}

#[tokio::test]
async fn error_when_transfer_nonexistent_token() -> Result<()> {
    let E2EContext { alice, bob } = E2EContext::<Erc721>::new().await?;
    let token_id = random_token_id();
    let err = alice
        .transfer_from(alice.wallet.address(), bob.wallet.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not transfer a non existent token");
    err.assert(ERC721NonexistentToken { token_id })
}

#[tokio::test]
async fn approve_token_transfer() -> Result<()> {
    let E2EContext { alice, bob } = E2EContext::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = alice.mint(alice.wallet.address(), token_id).ctx_send().await?;
    let _ = alice.approve(bob.wallet.address(), token_id).ctx_send().await?;
    let _ = bob
        .transfer_from(alice.wallet.address(), bob.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let owner = bob.owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, bob.wallet.address());
    Ok(())
}

#[tokio::test]
async fn error_when_transfer_unapproved_token() -> Result<()> {
    let E2EContext { alice, bob } = E2EContext::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = alice.mint(alice.wallet.address(), token_id).ctx_send().await?;
    let err = bob
        .transfer_from(alice.wallet.address(), bob.wallet.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not transfer unapproved token");
    err.assert(ERC721InsufficientApproval {
        operator: bob.wallet.address(),
        token_id,
    })
}

// TODO: add more tests for erc721
