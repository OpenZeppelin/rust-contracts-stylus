#![cfg(feature = "e2e")]

use abi::Erc721;
use alloy::{
    network::ReceiptResponse,
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use e2e::{receipt, watch, Account, EventExt, ReceiptExt, Revert};
use eyre::ContextCompat;

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
    base_uri: &str,
) -> eyre::Result<Address> {
    let args = Erc721MetadataExample::constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        baseUri_: base_uri.to_owned(),
    };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(rpc_url, private_key, Some(args)).await?.address()
}

// ============================================================================
// Integration Tests: ERC-721 Metadata Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let base_uri = "";

    let contract_addr = deploy(alice.url(), &alice.pk(), base_uri).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::nameReturn { name } = contract.name().call().await?;
    let Erc721::symbolReturn { symbol } = contract.symbol().call().await?;
    let Erc721::baseUriReturn { baseURI } = contract.baseUri().call().await?;

    assert_eq!(TOKEN_NAME.to_owned(), name);
    assert_eq!(TOKEN_SYMBOL.to_owned(), symbol);
    assert_eq!(base_uri.to_owned(), baseURI);

    Ok(())
}

#[e2e::test]
async fn constructs_with_base_uri(alice: Account) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus";

    let contract_addr = deploy(alice.url(), &alice.pk(), base_uri).await?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::baseUriReturn { baseURI } = contract.baseUri().call().await?;

    assert_eq!(base_uri.to_owned(), baseURI);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 URI Storage Extension
// ============================================================================

#[e2e::test]
async fn error_when_checking_token_uri_for_nonexistent_token(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = deploy(alice.url(), &alice.pk(), "").await?;

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
    let contract_addr = deploy(alice.url(), &alice.pk(), "").await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!("", tokenURI);

    Ok(())
}

#[e2e::test]
async fn return_token_uri_with_base_uri_and_without_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = deploy(alice.url(), &alice.pk(), base_uri).await?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri.to_owned() + &token_id.to_string(), tokenURI);
    Ok(())
}

#[e2e::test]
async fn return_token_uri_with_base_uri_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = deploy(alice.url(), &alice.pk(), base_uri).await?;

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

    assert_eq!(base_uri.to_owned() + &token_uri, tokenURI);

    Ok(())
}

#[e2e::test]
async fn set_token_uri_before_mint(alice: Account) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = deploy(alice.url(), &alice.pk(), base_uri).await?;

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

    assert_eq!(base_uri.to_owned() + &token_uri, tokenURI);

    Ok(())
}

#[e2e::test]
async fn return_token_uri_after_burn_and_remint(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let alice_addr = alice.address();

    let contract_addr = deploy(alice.url(), &alice.pk(), base_uri).await?;

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

    assert_eq!(base_uri.to_owned() + &token_id.to_string(), tokenURI);
    Ok(())
}
