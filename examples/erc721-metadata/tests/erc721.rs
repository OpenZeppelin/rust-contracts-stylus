#![cfg(feature = "e2e")]

use alloy::{
    primitives::{Address, U256},
    sol,
    sol_types::SolConstructor,
};
use e2e::{receipt, watch, Account, EventExt};

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
