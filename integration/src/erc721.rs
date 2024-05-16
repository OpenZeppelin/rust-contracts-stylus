use ethers::prelude::*;
use eyre::{bail, Result};

use crate::infrastructure::{erc721::*, *};

// TODO: add isolation with mutex per contract

#[tokio::test]
async fn mint() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let owner = infra.alice.owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, infra.alice.wallet.address());

    let balance =
        infra.alice.balance_of(infra.alice.wallet.address()).ctx_call().await?;
    assert!(balance >= U256::one());
    Ok(())
}

#[tokio::test]
async fn error_when_reusing_token_id() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let err = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not mint a token id twice");
    err.assert(ERC721InvalidSender { sender: Address::zero() })
}

#[tokio::test]
async fn transfer() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let _ = infra
        .alice
        .transfer_from(
            infra.alice.wallet.address(),
            infra.bob.wallet.address(),
            token_id,
        )
        .ctx_send()
        .await?;
    let owner = infra.bob.owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, infra.bob.wallet.address());
    Ok(())
}

#[tokio::test]
async fn error_when_transfer_nonexistent_token() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let err = infra
        .alice
        .transfer_from(
            infra.alice.wallet.address(),
            infra.bob.wallet.address(),
            token_id,
        )
        .ctx_send()
        .await
        .expect_err("should not transfer a non existent token");
    err.assert(ERC721NonexistentToken { token_id })
}

#[tokio::test]
async fn approve_token_transfer() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let _ = infra
        .alice
        .approve(infra.bob.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let _ = infra
        .bob
        .transfer_from(
            infra.alice.wallet.address(),
            infra.bob.wallet.address(),
            token_id,
        )
        .ctx_send()
        .await?;
    let owner = infra.bob.owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, infra.bob.wallet.address());
    Ok(())
}

#[tokio::test]
async fn error_when_transfer_unapproved_token() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    let err = infra
        .bob
        .transfer_from(
            infra.alice.wallet.address(),
            infra.bob.wallet.address(),
            token_id,
        )
        .ctx_send()
        .await
        .expect_err("should not transfer unapproved token");
    err.assert(ERC721InsufficientApproval {
        operator: infra.bob.wallet.address(),
        token_id,
    })
}

// TODO: add more tests for erc721
