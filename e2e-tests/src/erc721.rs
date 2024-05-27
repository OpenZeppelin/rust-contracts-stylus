use e2e_grip::prelude::*;

use crate::abi::erc721::*;

pub async fn mint() -> Result<()> {
    let alice = User::new().await?;
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let _ =
        alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
    let owner = alice.uses(erc721).owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, alice.address());

    let balance =
        alice.uses(erc721).balance_of(alice.address()).ctx_call().await?;
    assert!(balance >= U256::one());
    Ok(())
}

pub async fn error_when_reusing_token_id() -> Result<()> {
    let alice = User::new().await?;
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let _ =
        alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
    let err = alice
        .uses(erc721)
        .mint(alice.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not mint a token id twice");
    err.assert(ERC721InvalidSender { sender: Address::zero() })
}

pub async fn transfer() -> Result<()> {
    let alice = User::new().await?;
    let bob = User::new().await?;
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let _ =
        alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
    let _ = alice
        .uses(erc721)
        .transfer_from(alice.address(), bob.address(), token_id)
        .ctx_send()
        .await?;
    let owner = bob.uses(erc721).owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, bob.address());
    Ok(())
}

pub async fn error_when_transfer_nonexistent_token() -> Result<()> {
    let alice = User::new().await?;
    let bob = User::new().await?;
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let err = alice
        .uses(&erc721)
        .transfer_from(alice.address(), bob.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not transfer a non existent token");
    err.assert(ERC721NonexistentToken { token_id })
}

pub async fn approve_token_transfer() -> Result<()> {
    let alice = User::new().await?;
    let bob = User::new().await?;
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let _ =
        alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
    let _ =
        alice.uses(erc721).approve(bob.address(), token_id).ctx_send().await?;
    let _ = bob
        .uses(erc721)
        .transfer_from(alice.address(), bob.address(), token_id)
        .ctx_send()
        .await?;
    let owner = bob.uses(erc721).owner_of(token_id).ctx_call().await?;
    assert_eq!(owner, bob.address());
    Ok(())
}

pub async fn error_when_transfer_unapproved_token() -> Result<()> {
    let alice = User::new().await?;
    let bob = User::new().await?;
    let erc721 = &alice.deploys::<Erc721>().await?;
    let token_id = random_token_id();
    let _ =
        alice.uses(erc721).mint(alice.address(), token_id).ctx_send().await?;
    let err = bob
        .uses(erc721)
        .transfer_from(alice.address(), bob.address(), token_id)
        .ctx_send()
        .await
        .expect_err("should not transfer unapproved token");
    err.assert(ERC721InsufficientApproval { operator: bob.address(), token_id })
}

// TODO: add more tests for erc721
