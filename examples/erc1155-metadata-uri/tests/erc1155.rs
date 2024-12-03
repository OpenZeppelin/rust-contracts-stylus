#![cfg(feature = "e2e")]

use abi::Erc1155;
use alloy::{
    primitives::{fixed_bytes, uint, Address, U256},
    sol,
};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use mock::{receiver, receiver::ERC1155ReceiverMock};

use crate::Erc1155MetadataUriExample::constructorCall;

mod abi;
mod mock;

sol!("src/constructor.sol");

const URI: &str = "https://github.com/OpenZeppelin/rust-contracts-stylus";
const BASE_URI: &str = "https://github.com";
const EMPTY_BASE_URI: &str = "";

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(U256::from).collect()
}

fn random_values(size: usize) -> Vec<U256> {
    (1..size + 1).map(U256::from).collect()
}

fn ctr(uri: &str, base_uri: &str) -> constructorCall {
    constructorCall { uri_: uri.to_owned(), baseUri_: base_uri.to_owned() }
}

// ============================================================================
// Integration Tests: ERC-1155 URI Storage Extension
// ============================================================================

#[e2e::test]
async fn error_when_checking_token_uri_for_nonexistent_token(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let err = contract
        .tokenURI(token_id)
        .call()
        .await
        .expect_err("should return ERC1155NonexistentToken");

    assert!(err
        .reverted_with(Erc1155::ERC1155NonexistentToken { tokenId: token_id }));
    Ok(())
}

#[e2e::test]
async fn return_empty_token_uri_when_without_base_uri_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc1155::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!("", tokenURI);

    Ok(())
}

#[e2e::test]
async fn return_token_uri_with_base_uri_and_without_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc1155::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri.to_owned() + &token_id.to_string(), tokenURI);
    Ok(())
}

#[e2e::test]
async fn return_token_uri_with_base_uri_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let token_uri = String::from(
        "blob/main/contracts/src/token/erc1155/extensions/uri_storage.rs",
    );

    let receipt = receipt!(contract.setTokenURI(token_id, token_uri.clone()))?;

    assert!(receipt.emits(Erc1155::MetadataUpdate { tokenId: token_id }));

    let Erc1155::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri.to_owned() + &token_uri, tokenURI);

    Ok(())
}

#[e2e::test]
async fn set_token_uri_before_mint(alice: Account) -> eyre::Result<()> {
    let base_uri = "https://github.com/OpenZeppelin/rust-contracts-stylus/";

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC1155NonexistentToken`");

    assert!(err
        .reverted_with(Erc1155::ERC1155NonexistentToken { tokenId: token_id }));

    let token_uri = String::from(
        "blob/main/contracts/src/token/erc1155/extensions/uri_storage.rs",
    );

    let receipt = receipt!(contract.setTokenURI(token_id, token_uri.clone()))?;

    assert!(receipt.emits(Erc1155::MetadataUpdate { tokenId: token_id }));

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let Erc1155::tokenURIReturn { tokenURI } =
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

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(base_uri))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_id();

    let _ = watch!(contract.mint(alice.address(), token_id))?;

    let receipt = receipt!(contract.burn(token_id))?;

    assert!(receipt.emits(Erc1155::Transfer {
        from: alice_addr,
        to: Address::ZERO,
        tokenId: token_id,
    }));

    let err = contract
        .ownerOf(token_id)
        .call()
        .await
        .expect_err("should return `ERC1155NonexistentToken`");

    assert!(err
        .reverted_with(Erc1155::ERC1155NonexistentToken { tokenId: token_id }));

    let receipt = receipt!(contract.mint(alice_addr, token_id))?;

    assert!(receipt.emits(Erc1155::Transfer {
        from: Address::ZERO,
        to: alice_addr,
        tokenId: token_id
    }));

    let Erc1155::ownerOfReturn { ownerOf: owner_of } =
        contract.ownerOf(token_id).call().await?;
    assert_eq!(alice_addr, owner_of);

    let Erc1155::tokenURIReturn { tokenURI } =
        contract.tokenURI(token_id).call().await?;

    assert_eq!(base_uri.to_owned() + &token_id.to_string(), tokenURI);
    Ok(())
}

// ============================================================================
// Integration Tests: ERC-165 Support Interface
// ============================================================================

#[e2e::test]
async fn support_interface(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_default_constructor::<constructorCall>()
        .deploy()
        .await?
        .address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let invalid_interface_id: u32 = 0xffffffff;
    let supports_interface = contract
        .supportsInterface(invalid_interface_id.into())
        .call()
        .await?
        ._0;

    assert!(!supports_interface);

    let erc1155_interface_id: u32 = 0xd9b67a26;
    let supports_interface = contract
        .supportsInterface(erc1155_interface_id.into())
        .call()
        .await?
        ._0;

    assert!(supports_interface);

    let erc165_interface_id: u32 = 0x01ffc9a7;
    let supports_interface =
        contract.supportsInterface(erc165_interface_id.into()).call().await?._0;

    assert!(supports_interface);

    let erc1155_metadata_interface_id: u32 = 0x0e89341c;
    let supports_interface = contract
        .supportsInterface(erc1155_metadata_interface_id.into())
        .call()
        .await?
        ._0;

    assert!(supports_interface);

    Ok(())
}
