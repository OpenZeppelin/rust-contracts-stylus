#![cfg(feature = "e2e")]

use abi::Erc721HolderExample;
use alloy::primitives::{Bytes, U256};
use e2e::Account;
use eyre::Result;
use openzeppelin_stylus::token::erc721::RECEIVER_FN_SELECTOR;

mod abi;

#[e2e::test]
async fn returns_correct_selector(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc721HolderExample::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let token_id = U256::ONE;
    let data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
    let interface_selector = contract
        .onERC721Received(operator, from, token_id, data)
        .call()
        .await?
        ._0;

    assert_eq!(RECEIVER_FN_SELECTOR, interface_selector);

    Ok(())
}
