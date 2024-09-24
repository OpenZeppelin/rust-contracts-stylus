#![cfg(feature = "e2e")]

use abi::Erc1155;
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};
use stylus_sdk::contract;

mod abi;

// ============================================================================
// Integration Tests: ERC-1155 Token Standard
// ============================================================================

#[e2e::test]
async fn constructs_erc1155(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let Erc1155::pausedReturn { paused } = contract.paused().call().await?;

    assert_eq!(false, paused);

    Ok(())
}
