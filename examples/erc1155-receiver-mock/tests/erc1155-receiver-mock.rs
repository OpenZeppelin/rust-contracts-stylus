#![cfg(feature = "e2e")]

use abi::Erc1155ReceiverMock;
use alloy::primitives::{uint, Bytes, U256, U8};
use e2e::{constructor, Account, Constructor, Panic, PanicCode, Revert};
use eyre::Result;
use openzeppelin_stylus::token::erc1155::receiver::{
    BATCH_TRANSFER_FN_SELECTOR, SINGLE_TRANSFER_FN_SELECTOR,
};

mod abi;

const REVERT_TYPE_NONE: U8 = uint!(0_U8);
const REVERT_TYPE_CUSTOM_ERROR: U8 = uint!(1_U8);
const REVERT_TYPE_PANIC: U8 = uint!(2_U8);

fn constructor(error_type: U8) -> Constructor {
    constructor!(error_type)
}

// ============================================================================
// Happy Path Tests
// ============================================================================

#[e2e::test]
async fn returns_correct_selector_for_single_transfer(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_NONE))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let id = U256::from(1);
    let value = U256::from(1);
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_NONE))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let ids = vec![U256::from(1), U256::from(2)];
    let values = vec![U256::from(1), U256::from(2)];
    let data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
    let interface_selector = contract
        .onERC1155BatchReceived(operator, from, ids, values, data)
        .call()
        .await?
        ._0;

    assert_eq!(BATCH_TRANSFER_FN_SELECTOR, interface_selector);

    Ok(())
}

// ============================================================================
// Error Handling Tests - Single Transfer
// ============================================================================

#[e2e::test]
async fn reverts_without_message_for_single_transfer(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_CUSTOM_ERROR))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let id = U256::from(1);
    let value = U256::from(1);
    let data = Bytes::new();

    let err = contract
        .onERC1155Received(operator, from, id, value, data)
        .call()
        .await
        .expect_err("should revert with custom error");

    assert!(err.reverted_with(Erc1155ReceiverMock::CustomError {
        data: SINGLE_TRANSFER_FN_SELECTOR
    }));

    Ok(())
}

#[e2e::test]
async fn panics_for_single_transfer(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_PANIC))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let id = U256::from(1);
    let value = U256::from(1);
    let data = Bytes::new();

    let err = contract
        .onERC1155Received(operator, from, id, value, data)
        .call()
        .await
        .expect_err("should panic");

    assert!(err.panicked_with(PanicCode::DivisionByZero));

    Ok(())
}

// ============================================================================
// Error Handling Tests - Batch Transfer
// ============================================================================

#[e2e::test]
async fn reverts_with_custom_error_for_batch_transfer(
    alice: Account,
) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_CUSTOM_ERROR))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let ids = vec![U256::from(1), U256::from(2)];
    let values = vec![U256::from(1), U256::from(2)];
    let data = Bytes::new();

    let err = contract
        .onERC1155BatchReceived(operator, from, ids, values, data)
        .call()
        .await
        .expect_err("should revert with custom error");

    assert!(err.reverted_with(Erc1155ReceiverMock::CustomError {
        data: BATCH_TRANSFER_FN_SELECTOR
    }));

    Ok(())
}

#[e2e::test]
async fn panics_for_batch_transfer(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_PANIC))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let ids = vec![U256::from(1), U256::from(2)];
    let values = vec![U256::from(1), U256::from(2)];
    let data = Bytes::new();

    let err = contract
        .onERC1155BatchReceived(operator, from, ids, values, data)
        .call()
        .await
        .expect_err("should panic");

    assert!(err.panicked_with(PanicCode::DivisionByZero));

    Ok(())
}

// ============================================================================
// Interface Support Tests
// ============================================================================

#[e2e::test]
async fn supports_interface(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_NONE))
        .deploy()
        .await?
        .contract_address;
    let contract = Erc1155ReceiverMock::new(contract_addr, &alice.wallet);

    let invalid_interface_id: u32 = 0xffffffff;
    assert!(
        !contract
            .supportsInterface(invalid_interface_id.into())
            .call()
            .await?
            .supportsInterface
    );

    let erc1155_holder_interface_id: u32 = 0x4e2312e0;
    assert!(
        contract
            .supportsInterface(erc1155_holder_interface_id.into())
            .call()
            .await?
            .supportsInterface
    );

    let erc165_interface_id: u32 = 0x01ffc9a7;
    assert!(
        contract
            .supportsInterface(erc165_interface_id.into())
            .call()
            .await?
            .supportsInterface
    );

    Ok(())
}
