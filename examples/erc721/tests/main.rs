#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use e2e::{prelude::Assert, user::User};

use crate::abi::Erc721;

mod abi;

sol!("src/constructor.sol");

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(rpc_url: &str, private_key: &str) -> eyre::Result<Address> {
    let name = env!("CARGO_PKG_NAME").replace('-', "_");
    let pkg_dir = env!("CARGO_MANIFEST_DIR");
    let args = Erc721Example::constructorCall {
        name_: "Test Token".to_owned(),
        symbol_: "NFT".to_owned(),
    };
    let args = alloy::hex::encode(args.abi_encode());
    let contract_addr =
        e2e::deploy::deploy(&name, pkg_dir, rpc_url, private_key, Some(args))
            .await?;

    Ok(contract_addr)
}

macro_rules! send {
    ($e: expr) => {
        $e.send().await?.watch().await
    };
}

#[e2e::test]
async fn mint(alice: User) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.signer);

    let token_id = random_token_id();
    let _ = send!(contract.mint(alice.address(), token_id))?;
    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(ownerOf, alice.address());

    let Erc721::balanceOfReturn { balance } =
        contract.balanceOf(alice.address()).call().await?;
    assert!(balance >= U256::from(1));
    Ok(())
}

#[e2e::test]
async fn error_when_reusing_token_id(alice: User) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.signer);

    let token_id = random_token_id();
    let _ = send!(contract.mint(alice.address(), token_id))?;
    let err = send!(contract.mint(alice.address(), token_id))
        .expect_err("should not mint a token id twice");
    err.assert(Erc721::ERC721InvalidSender { sender: Address::ZERO });
    Ok(())
}

#[e2e::test]
async fn transfer(alice: User, bob: User) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.signer);

    let token_id = random_token_id();
    let _ = send!(contract.mint(alice.address(), token_id))?;
    let _ =
        send!(contract.transferFrom(alice.address(), bob.address(), token_id))?;

    let contract = Erc721::new(contract_addr, &bob.signer);
    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(ownerOf, bob.address());
    Ok(())
}

#[e2e::test]
async fn error_when_transfer_nonexistent_token(
    alice: User,
    bob: User,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.signer);

    let token_id = random_token_id();
    let tx = contract.transferFrom(alice.address(), bob.address(), token_id);
    let err = send!(tx).expect_err("should not transfer a non existent token");
    err.assert(Erc721::ERC721NonexistentToken { tokenId: token_id });
    Ok(())
}

#[e2e::test]
async fn approve_token_transfer(alice: User, bob: User) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.signer);

    let token_id = random_token_id();
    let _ = send!(contract.mint(alice.address(), token_id))?;
    let _ = send!(contract.approve(bob.address(), token_id))?;

    let contract = Erc721::new(contract_addr, &bob.signer);
    let _ =
        send!(contract.transferFrom(alice.address(), bob.address(), token_id))?;
    let Erc721::ownerOfReturn { ownerOf } =
        contract.ownerOf(token_id).call().await?;
    assert_ne!(ownerOf, alice.address());
    assert_eq!(ownerOf, bob.address());
    Ok(())
}

#[e2e::test]
async fn error_when_transfer_unapproved_token(
    alice: User,
    bob: User,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk()).await?;
    let contract = Erc721::new(contract_addr, &alice.signer);

    let token_id = random_token_id();
    let _ = send!(contract.mint(alice.address(), token_id))?;

    let contract = Erc721::new(contract_addr, &bob.signer);
    let err =
        send!(contract.transferFrom(alice.address(), bob.address(), token_id))
            .expect_err("should not transfer unapproved token");
    err.assert(Erc721::ERC721InsufficientApproval {
        operator: bob.address(),
        tokenId: token_id,
    });
    Ok(())
}
