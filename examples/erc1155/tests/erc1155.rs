#![cfg(feature = "e2e")]

use abi::Erc1155;
use alloy::primitives::{uint, Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};

mod abi;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(|_| U256::from(rand::random::<u32>())).collect()
}

fn random_values(size: usize) -> Vec<U256> {
    (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
}

// ============================================================================
// Integration Tests: ERC-1155 Token Standard
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let _contract = Erc1155::new(contract_addr, &alice.wallet);

    Ok(())
}

#[e2e::test]
async fn error_when_array_length_mismatch(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let token_ids = random_token_ids(1);

    let Erc1155::balanceOfReturn { balance } =
        contract.balanceOf(alice.address(), token_ids[0]).call().await?;
    assert_eq!(uint!(0_U256), balance);

    Ok(())
}

#[e2e::test]
async fn balance_of_batch_zero_balance(
    alice: Account,
    bob: Account,
    dave: Account,
    charlie: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);
    let accounts =
        vec![alice.address(), bob.address(), dave.address(), charlie.address()];
    let token_ids = random_token_ids(4);

    let Erc1155::balanceOfBatchReturn { balances } =
        contract.balanceOfBatch(accounts, token_ids).call().await?;
    assert_eq!(
        vec![uint!(0_U256), uint!(0_U256), uint!(0_U256), uint!(0_U256)],
        balances
    );

    Ok(())
}

#[e2e::test]
async fn set_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
async fn error_when_invalid_operator_is_approved_for_all_(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let invalid_operator = Address::ZERO;

    let Erc1155::isApprovedForAllReturn { approved } = contract
        .isApprovedForAll(alice.address(), invalid_operator)
        .call()
        .await?;

    assert_eq!(false, approved);

    Ok(())
}

#[e2e::test]
async fn mints(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
async fn mint_batch(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
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

    let receipt = receipt!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![0, 1, 2, 3].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: Address::ZERO,
        to: bob_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    for (token_id, value) in token_ids.iter().zip(values.iter()) {
        let Erc1155::balanceOfReturn { balance } =
            contract.balanceOf(bob_addr, *token_id).call().await?;
        assert_eq!(*value, balance);
    }

    let receipt = receipt!(contract.mintBatch(
        dave_addr,
        token_ids.clone(),
        values.clone(),
        vec![0, 1, 2, 3].into()
    ))?;

    assert!(receipt.emits(Erc1155::TransferBatch {
        operator: alice_addr,
        from: Address::ZERO,
        to: dave_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    for (token_id, value) in token_ids.iter().zip(values.iter()) {
        let Erc1155::balanceOfReturn { balance } =
            contract.balanceOf(dave_addr, *token_id).call().await?;
        assert_eq!(*value, balance);
    }

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    let _ = watch!(contract.mint(
        alice_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ));

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

    let Erc1155::balanceOfReturn { balance } =
        contract.balanceOf(bob_addr, token_id).call().await?;
    assert_eq!(value, balance);

    Ok(())
}

#[e2e::test]
async fn error_when_invalid_receiver_safe_transfer_from(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    let _ = watch!(contract.mint(alice_addr, token_id, value, vec![].into()));

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
async fn error_when_invalid_sender_safe_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let invalid_sender = Address::ZERO;
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    // let _ = watch!(contract.mint(alice_addr, token_id, value,
    // vec![].into()));
    let _ = watch!(contract.setOperatorApprovals(
        invalid_sender,
        alice.address(),
        true
    ));

    let err = send!(contract.safeTransferFrom(
        invalid_sender,
        bob_addr,
        token_id,
        value,
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidSender`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidSender {
        sender: invalid_sender
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_missing_approval_safe_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    let _ = watch!(contract.mint(bob_addr, token_id, value, vec![].into()));

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
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let _ = watch!(contract.mint(bob_addr, token_id, value, vec![].into()));
    let _ = watch!(contract.setOperatorApprovals(bob_addr, alice_addr, true));

    let err = send!(contract.safeTransferFrom(
        bob_addr,
        dave_addr,
        token_id,
        value + uint!(1_U256),
        vec![].into()
    ))
    .expect_err("should return `ERC1155InsufficientBalance`");

    assert!(err.reverted_with(Erc1155::ERC1155InsufficientBalance {
        sender: bob_addr,
        balance: value,
        needed: value + uint!(1_U256),
        id: token_id
    }));

    Ok(())
}

#[e2e::test]
async fn safe_batch_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);
    let amount_one = values[0] - uint!(1_U256);
    let amount_two = values[1] - uint!(1_U256);

    let _ = watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));
    let _ = watch!(contract.setOperatorApprovals(bob_addr, alice_addr, true));

    let receipt = receipt!(contract.safeBatchTransferFrom(
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

    let balance_id_one =
        contract.balanceOf(dave_addr, token_ids[0]).call().await?.balance;
    let balance_id_two =
        contract.balanceOf(dave_addr, token_ids[1]).call().await?.balance;

    assert_eq!(values[0], balance_id_one);
    assert_eq!(values[1], balance_id_two);

    Ok(())
}

#[e2e::test]
async fn error_when_invalid_receiver_safe_batch_transfer_from(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let invalid_receiver = Address::ZERO;
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));

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
async fn error_when_invalid_sender_safe_batch_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let invalid_sender = Address::ZERO;
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));
    let _ = watch!(contract.setOperatorApprovals(
        invalid_sender,
        alice.address(),
        true
    ));

    let err = send!(contract.safeBatchTransferFrom(
        invalid_sender,
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidSender`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidSender {
        sender: invalid_sender
    }));

    Ok(())
}

#[e2e::test]
async fn error_when_missing_approval_safe_batch_transfer_from(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(4);
    let values = random_values(4);

    let _ = watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));

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
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));
    let _ = watch!(contract.setOperatorApprovals(bob_addr, alice_addr, true));

    let err = send!(contract.safeBatchTransferFrom(
        bob_addr,
        dave_addr,
        token_ids.clone(),
        vec![values[0] + uint!(1_U256), values[1]],
        vec![].into()
    ))
    .expect_err("should return `ERC1155InsufficientBalance`");

    assert!(err.reverted_with(Erc1155::ERC1155InsufficientBalance {
        sender: bob_addr,
        balance: values[0],
        needed: values[0] + uint!(1_U256),
        id: token_ids[0]
    }));

    Ok(())
}
