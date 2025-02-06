#![cfg(feature = "e2e")]

use abi::Erc1155;
use alloy::{primitives::U256, sol};
use e2e::{receipt, watch, Account, EventExt, ReceiptExt};

use crate::Erc1155MetadataUriExample::constructorCall;

mod abi;

sol!("src/constructor.sol");

const URI: &str = "https://github.com/OpenZeppelin/rust-contracts-stylus";
const BASE_URI: &str = "https://github.com";

fn ctr(uri: &str) -> constructorCall {
    constructorCall { uri_: uri.to_owned() }
}

// ============================================================================
// Integration Tests: ERC-1155 URI Storage Extension
// ============================================================================

#[e2e::test]
async fn uri_returns_metadata_uri_when_token_uri_is_not_set(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(URI))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::from(1);

    let uri = contract.uri(token_id).call().await?.uri;

    assert_eq!(URI, uri);
    Ok(())
}

#[e2e::test]
async fn uri_returns_empty_string_when_no_uri_is_set(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::from(1);

    let uri = contract.uri(token_id).call().await?.uri;

    assert_eq!("", uri);

    Ok(())
}

#[e2e::test]
async fn uri_returns_concatenated_base_uri_and_token_uri(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::from(1);
    let token_uri = "/some/token/uri";
    let expected_uri = BASE_URI.to_owned() + token_uri;

    _ = watch!(contract.setBaseURI(BASE_URI.to_owned()))?;

    let receipt =
        receipt!(contract.setTokenURI(token_id, token_uri.to_owned()))?;

    assert!(receipt
        .emits(Erc1155::URI { value: expected_uri.clone(), id: token_id }));

    let uri = contract.uri(token_id).call().await?.uri;

    assert_eq!(expected_uri, uri);

    Ok(())
}

#[e2e::test]
async fn uri_returns_token_uri_when_base_uri_is_empty(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(""))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::from(1);
    let token_uri = "https://random.uri/some/token/uri";

    let receipt =
        receipt!(contract.setTokenURI(token_id, token_uri.to_owned()))?;

    assert!(receipt
        .emits(Erc1155::URI { value: token_uri.to_owned(), id: token_id }));

    let uri = contract.uri(token_id).call().await?.uri;

    assert_eq!(token_uri, uri);

    Ok(())
}

#[e2e::test]
async fn uri_ignores_metadata_uri_when_token_uri_is_set(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(URI))
        .deploy()
        .await?
        .address()?;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::from(1);
    let token_uri = "/some/token/uri";
    let expected_uri = BASE_URI.to_owned() + token_uri;

    _ = watch!(contract.setBaseURI(BASE_URI.to_owned()))?;

    let receipt =
        receipt!(contract.setTokenURI(token_id, token_uri.to_owned()))?;

    assert!(receipt
        .emits(Erc1155::URI { value: expected_uri.clone(), id: token_id }));

    let uri = contract.uri(token_id).call().await?.uri;

    assert_eq!(expected_uri, uri);

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-165 Support Interface
// ============================================================================

#[e2e::test]
async fn supports_interface(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(URI))
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
