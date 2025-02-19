#![cfg(feature = "e2e")]

use abi::Erc721;
use alloy::{
    primitives::{Address, U256},
    sol,
};
use e2e::{receipt, watch, Account, EventExt, ReceiptExt, Revert};

use crate::Erc721MetadataExample::constructorCall;

mod abi;

sol!("src/constructor.sol");

const TOKEN_NAME: &str = "Test Token";
const TOKEN_SYMBOL: &str = "NFT";

fn random_token_id() -> U256 {
    let num: u32 = rand::random();
    U256::from(num)
}

fn ctr(base_uri: &str) -> constructorCall {
    constructorCall {
        name_: TOKEN_NAME.to_owned(),
        symbol_: TOKEN_SYMBOL.to_owned(),
        baseUri_: base_uri.to_owned(),
    }
}

// ============================================================================
// Integration Tests: ERC-721 Metadata Extension
// ============================================================================

#[e2e::test]
async fn constructor_initializes_token_metadata_correctly(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(
            "https://github.com/OpenZeppelin/rust-contracts-stylus",
        ))
        .deploy()
        .await?
        .address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let Erc721::nameReturn { name } = contract.name().call().await?;
    let Erc721::symbolReturn { symbol } = contract.symbol().call().await?;

    assert_eq!(TOKEN_NAME.to_owned(), name);
    assert_eq!(TOKEN_SYMBOL.to_owned(), symbol);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-721 URI Storage Extension
// ============================================================================

#[e2e::test]
async fn token_uri_reverts_when_token_nonexistent(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

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
async fn token_uri_returns_empty_string_when_no_uri_set(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!("", tokenURI);

    Ok(())
}

#[e2e::test]
async fn token_uri_returns_base_uri_with_token_id_when_no_token_uri_set(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

    let contract = Erc721::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc721::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri.to_owned() + &token_id.to_string(), tokenURI);
    Ok(())
}

#[e2e::test]
async fn token_uri_returns_combined_base_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

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
async fn set_token_uri_succeeds_before_token_minting(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

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
async fn token_uri_remains_consistent_after_burn_and_remint(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let alice_addr = alice.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

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

// ============================================================================
// Integration Tests: ERC-165 Support Interface
// ============================================================================

#[e2e::test]
async fn supports_interface_returns_metadata_interface_id(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(
            "https://github.com/OpenZeppelin/rust-contracts-stylus",
        ))
        .deploy()
        .await?
        .address()?;
    let contract = Erc721::new(contract_addr, &alice.wallet);

    let erc721_metadata_interface_id: u32 = 0x5b5e139f;
    let supports_interface = contract
        .supportsInterface(erc721_metadata_interface_id.into())
        .call()
        .await?
        ._0;

    assert!(supports_interface);

    // other tests verify that other ERC-721 interfaces are implemented

    Ok(())
}
