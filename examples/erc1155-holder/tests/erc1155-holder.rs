#![cfg(feature = "e2e")]
#![allow(clippy::unreadable_literal)]

use abi::Erc1155HolderExample;
use alloy::primitives::{aliases::B32, uint, Bytes, U256};
use e2e::Account;
use eyre::Result;
use openzeppelin_stylus::token::erc1155::receiver::{
    BATCH_TRANSFER_FN_SELECTOR, SINGLE_TRANSFER_FN_SELECTOR,
};

mod abi;

#[e2e::test]
async fn returns_correct_selector_for_single_transfer(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc1155HolderExample::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let id = U256::ONE;
    let value = U256::ONE;
    let data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
    let interface_selector = contract
        .onERC1155Received(operator, from, id, value, data)
        .call()
        .await?
        ._0;

    assert_eq!(SINGLE_TRANSFER_FN_SELECTOR, interface_selector);

    Ok(())
}

#[e2e::test]
async fn returns_correct_selector_for_batch_transfer(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc1155HolderExample::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let ids = vec![U256::ONE, uint!(2_U256)];
    let values = vec![U256::ONE, uint!(2_U256)];
    let data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
    let interface_selector = contract
        .onERC1155BatchReceived(operator, from, ids, values, data)
        .call()
        .await?
        ._0;

    assert_eq!(BATCH_TRANSFER_FN_SELECTOR, interface_selector);

    Ok(())
}

#[e2e::test]
async fn supports_interface(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155HolderExample::new(contract_addr, &alice.wallet);

    let invalid_interface_id: B32 = 0xffffffff_u32.into();
    assert!(
        !contract
            .supportsInterface(invalid_interface_id)
            .call()
            .await?
            .supportsInterface
    );

    let erc1155_holder_interface_id: B32 = 0x4e2312e0_u32.into();
    assert!(
        contract
            .supportsInterface(erc1155_holder_interface_id)
            .call()
            .await?
            .supportsInterface
    );

    let erc165_interface_id: B32 = 0x01ffc9a7_u32.into();
    assert!(
        contract
            .supportsInterface(erc165_interface_id)
            .call()
            .await?
            .supportsInterface
    );

    Ok(())
}
