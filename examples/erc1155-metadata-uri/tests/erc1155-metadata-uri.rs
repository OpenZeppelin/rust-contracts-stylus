#![cfg(feature = "e2e")]

use abi::Erc1155;
use alloy::primitives::U256;
use e2e::{constructor, receipt, watch, Account, Constructor, EventExt};

mod abi;

const URI: &str = "https://github.com/OpenZeppelin/rust-contracts-stylus";
const BASE_URI: &str = "https://github.com";

fn ctr(uri: &str) -> Constructor {
    constructor!(uri.to_string())
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
        .contract_address;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::ONE;

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
        .contract_address;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::ONE;

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
        .contract_address;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::ONE;
    let token_uri = "/some/token/uri";
    let expected_uri = BASE_URI.to_owned() + token_uri;

    watch!(contract.setBaseURI(BASE_URI.to_owned()))?;

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
        .contract_address;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::ONE;
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
        .contract_address;

    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = U256::ONE;
    let token_uri = "/some/token/uri";
    let expected_uri = BASE_URI.to_owned() + token_uri;

    watch!(contract.setBaseURI(BASE_URI.to_owned()))?;

    let receipt =
        receipt!(contract.setTokenURI(token_id, token_uri.to_owned()))?;

    assert!(receipt
        .emits(Erc1155::URI { value: expected_uri.clone(), id: token_id }));

    let uri = contract.uri(token_id).call().await?.uri;

    assert_eq!(expected_uri, uri);

    Ok(())
}
