#![cfg(feature = "e2e")]

use abi::Erc1155;
use alloy::{
    primitives::{uint, Address, U256},
    sol_types::SolError,
};
use e2e::{receipt, send, watch, Account, EventExt, PanicCode, Revert};
use mock::{receiver, receiver::ERC1155ReceiverMock};

mod abi;
mod mock;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(U256::from).collect()
}

fn random_values(size: usize) -> Vec<U256> {
    (1..=size).map(U256::from).collect()
}

trait EncodeAsStr {
    fn encode_as_str(&self) -> String;
}

impl<T: SolError> EncodeAsStr for T {
    fn encode_as_str(&self) -> String {
        let expected_error = self.abi_encode();
        String::from_utf8_lossy(&expected_error).to_string()
    }
}

// ============================================================================
// Integration Tests: ERC-1155 Token
// ============================================================================

#[e2e::test]
async fn invalid_array_length_error_in_balance_of_batch(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_ids = random_token_ids(3);
    let accounts = vec![alice.address(), bob.address()];

    let err = contract
        .balanceOfBatch(accounts, token_ids)
        .call()
        .await
        .expect_err("should return `ERC1155InvalidArrayLength`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidArrayLength {
        idsLength: uint!(3_U256),
        valuesLength: uint!(2_U256)
    }));

    Ok(())
}

#[e2e::test]
async fn balance_of_zero_balance(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let token_ids = random_token_ids(1);

    let Erc1155::balanceOfReturn { balance } =
        contract.balanceOf(alice.address(), token_ids[0]).call().await?;
    assert_eq!(U256::ZERO, balance);

    Ok(())
}

#[e2e::test]
async fn balance_of_batch_zero_balance(
    alice: Account,
    bob: Account,
    dave: Account,
    charlie: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let accounts =
        vec![alice.address(), bob.address(), dave.address(), charlie.address()];
    let token_ids = random_token_ids(4);

    let Erc1155::balanceOfBatchReturn { balances } =
        contract.balanceOfBatch(accounts, token_ids).call().await?;
    assert_eq!(vec![U256::ZERO, U256::ZERO, U256::ZERO, U256::ZERO], balances);

    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let receipt = receipt!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: Address::ZERO,
        to: alice_addr,
        id: token_id,
        value
    }));

    let Erc1155::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(value, balance);

    Ok(())
}

#[e2e::test]
async fn mints_to_receiver_contract(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_addr =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let Erc1155::balanceOfReturn { balance: initial_receiver_balance } =
        contract.balanceOf(receiver_addr, token_id).call().await?;

    let receipt =
        receipt!(contract.mint(receiver_addr, token_id, value, vec![].into()))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: Address::ZERO,
        to: receiver_addr,
        id: token_id,
        value
    }));

    assert!(receipt.emits(ERC1155ReceiverMock::Received {
        operator: alice_addr,
        from: Address::ZERO,
        id: token_id,
        value,
        data: vec![].into(),
    }));

    let Erc1155::balanceOfReturn { balance: receiver_balance } =
        contract.balanceOf(receiver_addr, token_id).call().await?;
    assert_eq!(initial_receiver_balance + value, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_with_reason_in_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let err = send!(contract.mint(
        receiver_address,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))
    .expect_err("should not mint when receiver errors with reason");

    let message = Erc1155::Error {
        message: "ERC1155ReceiverMock: reverting on receive".to_string(),
    }
    .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_without_reason_in_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let err = send!(contract.mint(
        receiver_address,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))
    .expect_err("should not mint when receiver reverts");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: receiver_address
    }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_panics_in_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::Panic)
            .await?;

    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let err = send!(contract.mint(
        receiver_address,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))
    .expect_err("should not mint when receiver panics");

    let message =
        Erc1155::Panic { code: U256::from(PanicCode::DivisionByZero as u8) }
            .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_invalid_receiver_contract_in_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let err = send!(contract.mint(
        contract_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))
    .expect_err("should not mint when invalid receiver contract");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: contract_addr
    }));

    Ok(())
}

#[e2e::test]
async fn mint_batch(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(3);
    let values = random_values(3);

    let receipt = receipt!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![0, 1, 2, 3].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: Address::ZERO,
        to: alice_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    for (token_id, value) in token_ids.iter().zip(values.iter()) {
        let Erc1155::balanceOfReturn { balance } =
            contract.balanceOf(alice_addr, *token_id).call().await?;
        assert_eq!(*value, balance);
    }

    let Erc1155::balanceOfBatchReturn { balances } = contract
        .balanceOfBatch(
            vec![alice_addr, alice_addr, alice_addr],
            token_ids.clone(),
        )
        .call()
        .await?;

    assert_eq!(values, balances);
    Ok(())
}

#[e2e::test]
async fn mint_batch_transfer_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_addr =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let Erc1155::balanceOfBatchReturn { balances: initial_receiver_balances } =
        contract
            .balanceOfBatch(
                vec![receiver_addr, receiver_addr],
                token_ids.clone(),
            )
            .call()
            .await?;

    let receipt = receipt!(contract.mintBatch(
        receiver_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: Address::ZERO,
        to: receiver_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    assert!(receipt.emits(ERC1155ReceiverMock::BatchReceived {
        operator: alice_addr,
        from: Address::ZERO,
        ids: token_ids.clone(),
        values: values.clone(),
        data: vec![].into(),
    }));

    let Erc1155::balanceOfBatchReturn { balances: receiver_balances } =
        contract
            .balanceOfBatch(
                vec![receiver_addr, receiver_addr],
                token_ids.clone(),
            )
            .call()
            .await?;

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(
            initial_receiver_balances[idx] + value,
            receiver_balances[idx]
        );
    }

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_with_reason_in_batch_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let err = send!(contract.mintBatch(
        receiver_address,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should not mint batch when receiver errors with reason");

    let message = Erc1155::Error {
        message: "ERC1155ReceiverMock: reverting on batch receive".to_string(),
    }
    .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_without_reason_in_batch_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let err = send!(contract.mintBatch(
        receiver_address,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should not mint batch when receiver reverts");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: receiver_address
    }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_panics_in_batch_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::Panic)
            .await?;

    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let err = send!(contract.mintBatch(
        receiver_address,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should not mint batch when receiver panics");

    let message =
        Erc1155::Panic { code: U256::from(PanicCode::DivisionByZero as u8) }
            .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_invalid_receiver_contract_in_batch_mint(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let err = send!(contract.mintBatch(
        contract_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should not mint batch when invalid receiver contract");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: contract_addr,
    }));

    Ok(())
}

#[e2e::test]
async fn error_invalid_array_length_in_batch_mint(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);

    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let err = send!(contract_alice.mintBatch(
        bob_addr,
        vec![token_ids[0]],
        values,
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidArrayLength`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidArrayLength {
        idsLength: U256::ONE,
        valuesLength: uint!(2_U256)
    }));

    Ok(())
}

#[e2e::test]
async fn set_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let approved_value = true;
    let receipt =
        receipt!(contract.setApprovalForAll(bob_addr, approved_value))?;

    assert!(receipt.emits(Erc1155::ApprovalForAll {
        account: alice_addr,
        operator: bob_addr,
        approved: approved_value,
    }));

    let Erc1155::isApprovedForAllReturn { approved } =
        contract.isApprovedForAll(alice_addr, bob_addr).call().await?;
    assert_eq!(approved_value, approved);

    let approved_value = false;
    let receipt =
        receipt!(contract.setApprovalForAll(bob_addr, approved_value))?;

    assert!(receipt.emits(Erc1155::ApprovalForAll {
        account: alice_addr,
        operator: bob_addr,
        approved: approved_value,
    }));

    let Erc1155::isApprovedForAllReturn { approved } =
        contract.isApprovedForAll(alice_addr, bob_addr).call().await?;
    assert_eq!(approved_value, approved);

    Ok(())
}

#[e2e::test]
async fn error_when_invalid_operator_approval_for_all(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let invalid_operator = Address::ZERO;

    let err = send!(contract.setApprovalForAll(invalid_operator, true))
        .expect_err("should return `ERC1155InvalidOperator`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidOperator {
        operator: invalid_operator
    }));

    Ok(())
}

#[e2e::test]
async fn is_approved_for_all_zero_address(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let invalid_operator = Address::ZERO;

    let Erc1155::isApprovedForAllReturn { approved } = contract
        .isApprovedForAll(alice.address(), invalid_operator)
        .call()
        .await?;

    assert!(!approved);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    let Erc1155::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    let Erc1155::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr, token_id).call().await?;

    let receipt = receipt!(contract.safeTransferFrom(
        alice_addr,
        bob_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: bob_addr,
        id: token_id,
        value
    }));

    let Erc1155::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance - value, alice_balance);

    let Erc1155::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr, token_id).call().await?;
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from_with_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract_bob.mint(
        bob_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let Erc1155::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, token_id).call().await?;
    let Erc1155::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, token_id).call().await?;

    let receipt = receipt!(contract_alice.safeTransferFrom(
        bob_addr,
        alice_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: bob_addr,
        to: alice_addr,
        id: token_id,
        value
    }));

    let Erc1155::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance + value, alice_balance);

    let Erc1155::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, token_id).call().await?;
    assert_eq!(initial_bob_balance - value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_addr =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    let Erc1155::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    let Erc1155::balanceOfReturn { balance: initial_receiver_balance } =
        contract.balanceOf(receiver_addr, token_id).call().await?;

    let receipt = receipt!(contract.safeTransferFrom(
        alice_addr,
        receiver_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: receiver_addr,
        id: token_id,
        value
    }));

    assert!(receipt.emits(ERC1155ReceiverMock::Received {
        operator: alice_addr,
        from: alice_addr,
        id: token_id,
        value,
        data: vec![].into(),
    }));

    let Erc1155::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance - value, alice_balance);

    let Erc1155::balanceOfReturn { balance: receiver_balance } =
        contract.balanceOf(receiver_addr, token_id).call().await?;
    assert_eq!(initial_receiver_balance + value, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_with_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    let err = send!(contract.safeTransferFrom(
        alice_addr,
        receiver_address,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should not transfer when receiver errors with reason");

    let message = Erc1155::Error {
        message: "ERC1155ReceiverMock: reverting on receive".to_string(),
    }
    .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_without_reason(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    let err = send!(contract.safeTransferFrom(
        alice_addr,
        receiver_address,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should not transfer when receiver reverts");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: receiver_address
    }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_panics(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::Panic)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    let err = send!(contract.safeTransferFrom(
        alice_addr,
        receiver_address,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should not transfer when receiver panics");

    let message =
        Erc1155::Panic { code: U256::from(PanicCode::DivisionByZero as u8) }
            .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_invalid_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ))?;

    let err = send!(contract.safeTransferFrom(
        alice_addr,
        contract_addr,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should not transfer when invalid receiver contract");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: contract_addr,
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_invalid_receiver_safe_transfer_from(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    watch!(contract.mint(alice_addr, token_id, value, vec![].into()))?;

    let err = send!(contract.safeTransferFrom(
        alice_addr,
        invalid_receiver,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidReceiver`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: invalid_receiver
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_missing_approval_safe_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    watch!(contract.mint(bob_addr, token_id, value, vec![].into()))?;

    let err = send!(contract.safeTransferFrom(
        bob_addr,
        dave_addr,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should return `ERC1155MissingApprovalForAll`");

    assert!(err.reverted_with(Erc1155::ERC1155MissingApprovalForAll {
        operator: alice_addr,
        owner: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_insufficient_balance_safe_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract_alice.mint(bob_addr, token_id, value, vec![].into()))?;
    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let err = send!(contract_alice.safeTransferFrom(
        bob_addr,
        dave_addr,
        token_id,
        value + U256::ONE,
        vec![].into()
    ))
    .expect_err("should return `ERC1155InsufficientBalance`");

    assert!(err.reverted_with(Erc1155::ERC1155InsufficientBalance {
        sender: bob_addr,
        balance: value,
        needed: value + U256::ONE,
        tokenId: token_id
    }));

    Ok(())
}

#[e2e::test]
async fn safe_batch_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract_alice.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let Erc1155::balanceOfBatchReturn { balances: initial_alice_balances } =
        contract_alice
            .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155::balanceOfBatchReturn { balances: initial_bob_balances } =
        contract_alice
            .balanceOfBatch(vec![bob_addr, bob_addr], token_ids.clone())
            .call()
            .await?;

    let receipt = receipt!(contract_alice.safeBatchTransferFrom(
        alice_addr,
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: alice_addr,
        to: bob_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    let Erc1155::balanceOfBatchReturn { balances: alice_balances } =
        contract_alice
            .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155::balanceOfBatchReturn { balances: bob_balances } =
        contract_alice
            .balanceOfBatch(vec![bob_addr, bob_addr], token_ids.clone())
            .call()
            .await?;

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(initial_alice_balances[idx] - value, alice_balances[idx]);
        assert_eq!(initial_bob_balances[idx] + value, bob_balances[idx]);
    }

    Ok(())
}

#[e2e::test]
async fn safe_batch_transfer_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_addr =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let Erc1155::balanceOfBatchReturn { balances: initial_alice_balances } =
        contract
            .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155::balanceOfBatchReturn { balances: initial_receiver_balances } =
        contract
            .balanceOfBatch(
                vec![receiver_addr, receiver_addr],
                token_ids.clone(),
            )
            .call()
            .await?;

    let receipt = receipt!(contract.safeBatchTransferFrom(
        alice_addr,
        receiver_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: alice_addr,
        to: receiver_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    assert!(receipt.emits(ERC1155ReceiverMock::BatchReceived {
        operator: alice_addr,
        from: alice_addr,
        ids: token_ids.clone(),
        values: values.clone(),
        data: vec![].into(),
    }));

    let Erc1155::balanceOfBatchReturn { balances: alice_balances } = contract
        .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
        .call()
        .await?;

    let Erc1155::balanceOfBatchReturn { balances: receiver_balances } =
        contract
            .balanceOfBatch(
                vec![receiver_addr, receiver_addr],
                token_ids.clone(),
            )
            .call()
            .await?;

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(initial_alice_balances[idx] - value, alice_balances[idx]);
        assert_eq!(
            initial_receiver_balances[idx] + value,
            receiver_balances[idx]
        );
    }

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_with_reason_in_batch_transfer(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.safeBatchTransferFrom(
        alice_addr,
        receiver_address,
        token_ids,
        values,
        vec![].into()
    ))
    .expect_err("should not transfer when receiver errors with reason");

    let message = Erc1155::Error {
        message: "ERC1155ReceiverMock: reverting on batch receive".to_string(),
    }
    .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_reverts_without_reason_in_batch_transfer(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address = receiver::deploy(
        &alice.wallet,
        ERC1155ReceiverMock::RevertType::RevertWithoutMessage,
    )
    .await?;

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.safeBatchTransferFrom(
        alice_addr,
        receiver_address,
        token_ids,
        values,
        vec![].into()
    ))
    .expect_err("should not transfer when receiver reverts");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: receiver_address
    }));

    Ok(())
}

#[e2e::test]
async fn errors_when_receiver_panics_in_batch_transfer(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let receiver_address =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::Panic)
            .await?;

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.safeBatchTransferFrom(
        alice_addr,
        receiver_address,
        token_ids,
        values,
        vec![].into()
    ))
    .expect_err("should not transfer when receiver panics");

    let message =
        Erc1155::Panic { code: U256::from(PanicCode::DivisionByZero as u8) }
            .encode_as_str();

    assert!(err.reverted_with(Erc1155::InvalidReceiverWithReason { message }));

    Ok(())
}

#[e2e::test]
async fn errors_when_invalid_receiver_contract_in_batch_transfer(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.safeBatchTransferFrom(
        alice_addr,
        contract_addr,
        token_ids,
        values,
        vec![].into()
    ))
    .expect_err("should not transfer when invalid receiver contract");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: contract_addr,
    }));

    Ok(())
}

#[e2e::test]
async fn safe_batch_transfer_from_with_approval(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract_alice.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let Erc1155::balanceOfBatchReturn { balances: initial_dave_balances } =
        contract_alice
            .balanceOfBatch(vec![dave_addr, dave_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155::balanceOfBatchReturn { balances: initial_bob_balances } =
        contract_alice
            .balanceOfBatch(vec![bob_addr, bob_addr], token_ids.clone())
            .call()
            .await?;

    let receipt = receipt!(contract_alice.safeBatchTransferFrom(
        bob_addr,
        dave_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: bob_addr,
        to: dave_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    let Erc1155::balanceOfBatchReturn { balances: bob_balances } =
        contract_alice
            .balanceOfBatch(vec![bob_addr, bob_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155::balanceOfBatchReturn { balances: dave_balances } =
        contract_alice
            .balanceOfBatch(vec![dave_addr, dave_addr], token_ids.clone())
            .call()
            .await?;

    for (idx, value) in values.iter().enumerate() {
        assert_eq!(initial_bob_balances[idx] - value, bob_balances[idx]);
        assert_eq!(initial_dave_balances[idx] + value, dave_balances[idx]);
    }

    Ok(())
}

#[e2e::test]
async fn error_when_invalid_receiver_safe_batch_transfer_from(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.safeBatchTransferFrom(
        alice_addr,
        invalid_receiver,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidReceiver`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidReceiver {
        receiver: invalid_receiver
    }));

    Ok(())
}

#[e2e::test]
async fn error_invalid_array_length_in_safe_batch_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract_alice.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract_alice.safeBatchTransferFrom(
        alice_addr,
        bob_addr,
        vec![token_ids[0]],
        values.clone(),
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidArrayLength`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidArrayLength {
        idsLength: U256::ONE,
        valuesLength: uint!(2_U256)
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_missing_approval_safe_batch_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(4);
    let values = random_values(4);

    watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.safeBatchTransferFrom(
        bob_addr,
        dave_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should return `ERC1155MissingApprovalForAll`");

    assert!(err.reverted_with(Erc1155::ERC1155MissingApprovalForAll {
        operator: alice_addr,
        owner: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_insufficient_balance_safe_batch_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract_alice.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;
    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let err = send!(contract_alice.safeBatchTransferFrom(
        bob_addr,
        dave_addr,
        token_ids.clone(),
        vec![values[0] + U256::ONE, values[1]],
        vec![].into()
    ))
    .expect_err("should return `ERC1155InsufficientBalance`");

    assert!(err.reverted_with(Erc1155::ERC1155InsufficientBalance {
        sender: bob_addr,
        balance: values[0],
        needed: values[0] + U256::ONE,
        tokenId: token_ids[0]
    }));

    Ok(())
}

// ============================================================================
// Integration Tests: ERC-1155 Burnable Extension
// ============================================================================

#[e2e::test]
async fn burns(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(1);
    let values = random_values(1);

    watch!(contract.mint(alice_addr, token_ids[0], values[0], vec![].into()))?;

    let initial_balance =
        contract.balanceOf(alice_addr, token_ids[0]).call().await?.balance;
    assert_eq!(values[0], initial_balance);

    let receipt = receipt!(contract.burn(alice_addr, token_ids[0], values[0]))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: Address::ZERO,
        id: token_ids[0],
        value: values[0],
    }));

    let balance =
        contract.balanceOf(alice_addr, token_ids[0]).call().await?.balance;
    assert_eq!(U256::ZERO, balance);

    Ok(())
}

#[e2e::test]
async fn burns_with_approval(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(1);
    let values = random_values(1);

    watch!(contract.mint(bob_addr, token_ids[0], values[0], vec![].into()))?;

    let initial_balance =
        contract.balanceOf(bob_addr, token_ids[0]).call().await?.balance;
    assert_eq!(values[0], initial_balance);

    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let receipt = receipt!(contract.burn(bob_addr, token_ids[0], values[0]))?;

    assert!(receipt.emits(Erc1155::TransferSingle {
        operator: alice_addr,
        from: bob_addr,
        to: Address::ZERO,
        id: token_ids[0],
        value: values[0],
    }));

    let balance =
        contract.balanceOf(bob_addr, token_ids[0]).call().await?.balance;
    assert_eq!(U256::ZERO, balance);

    Ok(())
}

#[e2e::test]
async fn error_when_missing_approval_burn(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(1);
    let values = random_values(1);

    watch!(contract.mint(bob_addr, token_ids[0], values[0], vec![].into()))?;

    let err = send!(contract.burn(bob_addr, token_ids[0], values[0]))
        .expect_err("should return `ERC1155MissingApprovalForAll`");

    assert!(err.reverted_with(Erc1155::ERC1155MissingApprovalForAll {
        operator: alice_addr,
        owner: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_insufficient_balance_burn(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    let to_burn = value + U256::ONE;

    watch!(contract.mint(alice_addr, token_id, value, vec![].into()))?;

    let err = send!(contract.burn(alice_addr, token_id, to_burn))
        .expect_err("should return `ERC1155InsufficientBalance`");

    assert!(err.reverted_with(Erc1155::ERC1155InsufficientBalance {
        sender: alice_addr,
        balance: value,
        needed: to_burn,
        tokenId: token_id
    }));

    Ok(())
}

#[e2e::test]
async fn burns_batch(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(4);
    let values = random_values(4);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    for (&id, &value) in token_ids.iter().zip(values.iter()) {
        let balance = contract.balanceOf(alice_addr, id).call().await?.balance;
        assert_eq!(value, balance);
    }

    let receipt = receipt!(contract.burnBatch(
        alice_addr,
        token_ids.clone(),
        values.clone()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: alice_addr,
        to: Address::ZERO,
        ids: token_ids.clone(),
        values,
    }));

    for id in token_ids {
        let balance = contract.balanceOf(alice_addr, id).call().await?.balance;
        assert_eq!(U256::ZERO, balance);
    }

    Ok(())
}

#[e2e::test]
async fn burns_batch_with_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(4);
    let values = random_values(4);

    watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    for (&id, &value) in token_ids.iter().zip(values.iter()) {
        let balance = contract.balanceOf(bob_addr, id).call().await?.balance;
        assert_eq!(value, balance);
    }

    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let receipt = receipt!(contract.burnBatch(
        bob_addr,
        token_ids.clone(),
        values.clone()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: bob_addr,
        to: Address::ZERO,
        ids: token_ids.clone(),
        values,
    }));

    for id in token_ids {
        let balance = contract.balanceOf(bob_addr, id).call().await?.balance;
        assert_eq!(U256::ZERO, balance);
    }

    Ok(())
}

#[e2e::test]
async fn error_when_missing_approval_burn_batch(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.burnBatch(bob_addr, token_ids, values))
        .expect_err("should return `ERC1155MissingApprovalForAll`");

    assert!(err.reverted_with(Erc1155::ERC1155MissingApprovalForAll {
        operator: alice_addr,
        owner: bob_addr
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_insufficient_balance_burn_batch(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);
    let to_burn: Vec<U256> = values.iter().map(|v| v + U256::ONE).collect();

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let err = send!(contract.burnBatch(
        alice_addr,
        token_ids.clone(),
        to_burn.clone()
    ))
    .expect_err("should return `ERC1155InsufficientBalance`");

    assert!(err.reverted_with(Erc1155::ERC1155InsufficientBalance {
        sender: alice_addr,
        balance: values[0],
        needed: to_burn[0],
        tokenId: token_ids[0]
    }));

    Ok(())
}
