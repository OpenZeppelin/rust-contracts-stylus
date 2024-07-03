#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use e2e::{receipt, watch, Account, EventExt, Revert};

use crate::abi::Erc721;

mod abi;

sol!("src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "NFT";

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

async fn deploy(
    rpc_url: &str,
    private_key: &str,
    base_uri: &String,
) -> eyre::Result<Address> {
    let args = Erc721MetadataExample::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        baseUri_: base_uri.to_string(),
    };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await
}

// ============================================================================
// Integration Tests: ERC-721 Metadata Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let base_uri = String::new();

    let contract_addr = deploy(alice.url(), &alice.pk(), &base_uri).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::nameReturn { name } = contract.name().call().await?;
    let Erc721::symbolReturn { symbol } = contract.symbol().call().await?;
    let Erc721::baseUriReturn { baseURI } = contract.baseUri().call().await?;

    assert_eq!(TOKEN_NAME.to_owned(), name);
    assert_eq!(TOKEN_SYMBOL.to_owned(), symbol);
    assert_eq!(base_uri, baseURI);

    Ok(())
}

#[e2e::test]
async fn constructs_with_base_uri(alice: Account) -> eyre::Result<()> {
    let base_uri =
        String::from("https://github.com/OpenZeppelin/rust-contracts-stylus");

    let contract_addr = deploy(alice.url(), &alice.pk(), &base_uri).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::baseUriReturn { baseURI } = contract.baseUri().call().await?;

    assert_eq!(base_uri, baseURI);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 URI Storage Extension
// ============================================================================

#[e2e::test]
async fn error_when_token_uri_for_non_existent_token(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr =
        deploy(alice.url(), &alice.pk(), &String::new()).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let err = contract
        .tokenURI(token_id)
        .call()
        .await
        .expect_err("should return ERC721NonexistentToken");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );
    Ok(())
}

#[e2e::test]
async fn return_empty_token_uri_when_without_base_uri_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr =
        deploy(alice.url(), &alice.pk(), &String::new()).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(String::new(), tokenURI);

    Ok(())
}

#[e2e::test]
async fn return_token_uri_with_base_uri_and_without_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri =
        String::from("https://github.com/OpenZeppelin/rust-contracts-stylus/");

    let contract_addr = deploy(alice.url(), &alice.pk(), &base_uri).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri + &token_id.to_string(), tokenURI);
    Ok(())
}

#[e2e::test]
async fn return_token_uri_with_base_uri_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri =
        String::from("https://github.com/OpenZeppelin/rust-contracts-stylus/");

    let contract_addr = deploy(alice.url(), &alice.pk(), &base_uri).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let token_uri = String::from(
        "blob/main/contracts/src/token/erc721/extensions/uri_storage.rs",
    );

    let receipt = receipt!(contract.setTokenURI(token_id, token_uri.clone()))?;

    assert!(receipt.emits(Erc721::MetadataUpdate { tokenId: token_id }));

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri + &token_uri, tokenURI);

    Ok(())
}

#[e2e::test]
async fn set_token_uri_before_mint(alice: Account) -> eyre::Result<()> {
    let base_uri =
        String::from("https://github.com/OpenZeppelin/rust-contracts-stylus/");

    let contract_addr = deploy(alice.url(), &alice.pk(), &base_uri).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let token_uri = String::from(
        "blob/main/contracts/src/token/erc721/extensions/uri_storage.rs",
    );

    let receipt = receipt!(contract.setTokenURI(token_id, token_uri.clone()))?;

    assert!(receipt.emits(Erc721::MetadataUpdate { tokenId: token_id }));

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri + &token_uri, tokenURI);

    Ok(())
}

#[e2e::test]
async fn return_token_uri_after_burn_and_remint(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri =
        String::from("https://github.com/OpenZeppelin/rust-contracts-stylus/");

    let alice_addr = alice.address();

    let contract_addr = deploy(alice.url(), &alice.pk(), &base_uri).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let receipt = receipt!(contract.burn(token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        tokenId: token_id,
    }));

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC721NonexistentToken`");

    assert!(
        err.reverted_with(Erc721::ERC721NonexistentToken { tokenId: token_id })
    );

    let receipt = receipt!(contract.mint(alice_addr, token_id))?;

    assert!(receipt.emits(Erc721::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        tokenId: token_id
    }));

    let Erc721::ownerOfReturn { ownerOf: owner_of } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, owner_of);

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri + &token_id.to_string(), tokenURI);
    Ok(())
}
