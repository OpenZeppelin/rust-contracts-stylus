#![cfg(feature = "e2e")]

use abi::Erc721ReceiverMock;
use alloy::primitives::{uint, Bytes, U256, U8};
use e2e::{constructor, Account, Constructor, Panic, PanicCode, Revert};
use eyre::Result;
use openzeppelin_stylus::token::erc721::receiver::RECEIVER_FN_SELECTOR;

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
async fn returns_correct_selector(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_NONE))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc721ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let token_id = U256::from(1);
    let data = Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]);
    let interface_selector = contract
        .onERC721Received(operator, from, token_id, data)
        .call()
        .await?
        ._0;

    assert_eq!(RECEIVER_FN_SELECTOR, interface_selector);

    Ok(())
}

// ============================================================================
// Error Handling Tests - Single Transfer
// ============================================================================

#[e2e::test]
async fn reverts_without_message(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_CUSTOM_ERROR))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc721ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let token_id = U256::from(1);
    let data = Bytes::new();

    let err = contract
        .onERC721Received(operator, from, token_id, data)
        .call()
        .await
        .expect_err("should revert with custom error");

    assert!(err.reverted_with(Erc721ReceiverMock::CustomError {
        data: RECEIVER_FN_SELECTOR
    }));

    Ok(())
}

#[e2e::test]
async fn panics(alice: Account) -> Result<()> {
    let contract_addr = alice
        .as_deployer()
        .with_constructor(constructor(REVERT_TYPE_PANIC))
        .deploy()
        .await?
        .contract_address;

    let contract = Erc721ReceiverMock::new(contract_addr, &alice.wallet);

    let operator = alice.address();
    let from = alice.address();
    let token_id = U256::from(1);
    let data = Bytes::new();

    let err = contract
        .onERC721Received(operator, from, token_id, data)
        .call()
        .await
        .expect_err("should panic");

    assert!(err.panicked_with(PanicCode::DivisionByZero));

    Ok(())
}
