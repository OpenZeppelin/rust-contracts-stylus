use ethers::prelude::*;
use eyre::{bail, Result};

use crate::infrastructure::{erc721::*, *};

#[tokio::test]
async fn mint_nft_and_check_balance() -> Result<()> {
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
async fn error_mint_second_nft() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    match infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await
    {
        Ok(_) => {
            bail!("Second mint of the same token should not be possible")
        }
        Err(e) => e.assert(ERC721InvalidSender { sender: Address::zero() }),
    }
}

#[tokio::test]
async fn transfer_nft() -> Result<()> {
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
async fn error_transfer_nonexistent_nft() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    match infra
        .alice
        .transfer_from(
            infra.alice.wallet.address(),
            infra.bob.wallet.address(),
            token_id,
        )
        .ctx_send()
        .await
    {
        Ok(_) => {
            bail!("Transfer of a non existent nft should not be possible")
        }
        Err(e) => e.assert(ERC721NonexistentToken { token_id }),
    }
}

#[tokio::test]
async fn approve_nft_transfer() -> Result<()> {
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
async fn error_not_approved_nft_transfer() -> Result<()> {
    let infra = Infrastructure::<Erc721>::new().await?;
    let token_id = random_token_id();
    let _ = infra
        .alice
        .mint(infra.alice.wallet.address(), token_id)
        .ctx_send()
        .await?;
    match infra
        .bob
        .transfer_from(
            infra.alice.wallet.address(),
            infra.bob.wallet.address(),
            token_id,
        )
        .ctx_send()
        .await
    {
        Ok(_) => {
            bail!("Transfer of not approved token should not happen")
        }
        Err(e) => e.assert(ERC721InsufficientApproval {
            operator: infra.bob.wallet.address(),
            token_id,
        }),
    }
}
