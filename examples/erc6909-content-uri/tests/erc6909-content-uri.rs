#![cfg(feature = "e2e")]
#![allow(clippy::unreadable_literal)]

use abi::Erc6909ContentUri;
use alloy::primitives::{uint, U256};
use e2e::{receipt, send, Account, EventExt};
use eyre::Result;

mod abi;

// ============================================================================
// Integration Tests: ERC-6909 Content URI Extension
// ============================================================================

#[e2e::test]
async fn contract_uri_initially_empty(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909ContentUri::new(contract_addr, &alice.wallet);

    let Erc6909ContentUri::contractURIReturn { contractURI } =
        contract.contractURI().call().await?;

    assert!(contractURI.is_empty());
    Ok(())
}

#[e2e::test]
async fn token_uri_initially_empty(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909ContentUri::new(contract_addr, &alice.wallet);
    let id = uint!(1_U256);

    let Erc6909ContentUri::tokenURIReturn { tokenURI } =
        contract.tokenURI(id).call().await?;

    assert!(tokenURI.is_empty());
    Ok(())
}

#[e2e::test]
async fn sets_contract_uri(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909ContentUri::new(contract_addr, &alice.wallet);

    let uri = "https://example.com/contract".to_string();

    let receipt = receipt!(contract.setContractUri(uri.clone()))?;
    assert!(receipt.emits(Erc6909ContentUri::ContractURIUpdated {}));

    let Erc6909ContentUri::contractURIReturn { contractURI } =
        contract.contractURI().call().await?;

    assert_eq!(uri, contractURI);
    Ok(())
}

#[e2e::test]
async fn sets_token_uri(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909ContentUri::new(contract_addr, &alice.wallet);

    let id = uint!(1_U256);
    let uri = "https://example.com/token/1".to_string();

    let receipt = receipt!(contract.setTokenUri(id, uri.clone()))?;
    assert!(receipt.emits(Erc6909ContentUri::URI { value: uri.clone(), id }));

    let Erc6909ContentUri::tokenURIReturn { tokenURI } =
        contract.tokenURI(id).call().await?;

    assert_eq!(uri, tokenURI);
    Ok(())
}

#[e2e::test]
async fn sets_different_uris_per_token_id(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909ContentUri::new(contract_addr, &alice.wallet);

    let id1 = uint!(1_U256);
    let id2 = uint!(2_U256);
    let uri1 = "https://example.com/token/1".to_string();
    let uri2 = "https://example.com/token/2".to_string();

    let _ = receipt!(contract.setTokenUri(id1, uri1.clone()))?;
    let _ = receipt!(contract.setTokenUri(id2, uri2.clone()))?;

    let Erc6909ContentUri::tokenURIReturn { tokenURI: result1 } =
        contract.tokenURI(id1).call().await?;

    let Erc6909ContentUri::tokenURIReturn { tokenURI: result2 } =
        contract.tokenURI(id2).call().await?;

    assert_eq!(uri1, result1);
    assert_eq!(uri2, result2);
    Ok(())
}

#[e2e::test]
async fn mints_and_checks_balance(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909ContentUri::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let id = uint!(1_U256);
    let amount = uint!(100_U256);

    let _ = receipt!(contract.mint(alice_addr, id, amount))?;

    let Erc6909ContentUri::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    assert_eq!(amount, balance);
    Ok(())
}
