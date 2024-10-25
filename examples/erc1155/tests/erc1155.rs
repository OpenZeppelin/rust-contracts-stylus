#![cfg(feature = "e2e")]

use abi::Erc1155;
use alloy::primitives::{uint, Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, ReceiptExt, Revert};

mod abi;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(U256::from).collect()
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
async fn invalid_array_length_error_in_balance_of_batch(
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

    let accounts = vec![alice_addr, bob_addr, dave_addr];

    for account in accounts {
        let receipt = receipt!(contract.mintBatch(
            account,
            token_ids.clone(),
            values.clone(),
            vec![0, 1, 2, 3].into()
        ))?;

        assert!(receipt.emits(Erc1155::TransferBatch {
            operator: alice_addr,
            from: Address::ZERO,
            to: account,
            ids: token_ids.clone(),
            values: values.clone()
        }));

        for (token_id, value) in token_ids.iter().zip(values.iter()) {
            let Erc1155::balanceOfReturn { balance } =
                contract.balanceOf(account, *token_id).call().await?;
            assert_eq!(*value, balance);
        }

        let Erc1155::balanceOfBatchReturn { balances } = contract
            .balanceOfBatch(vec![account, account, account], token_ids.clone())
            .call()
            .await?;

        assert_eq!(values, balances);
    }
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
async fn is_approved_for_all_zero_address(alice: Account) -> eyre::Result<()> {
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
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let _ = watch!(contract_bob.mint(
        bob_addr,
        token_id,
        value,
        vec![0, 1, 2, 3].into()
    ));

    let _ = watch!(contract_bob.setApprovalForAll(alice_addr, true));

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
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let _ =
        watch!(contract_alice.mint(bob_addr, token_id, value, vec![].into()));
    let _ = watch!(contract_bob.setApprovalForAll(alice_addr, true));

    let err = send!(contract_alice.safeTransferFrom(
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
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract_alice.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));

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
async fn safe_batch_transfer_from_with_approval(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract_alice.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));

    let _ = watch!(contract_bob.setApprovalForAll(alice_addr, true));

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
async fn error_invalid_array_length_in_safe_batch_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract_alice.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));

    let err = send!(contract_alice.safeBatchTransferFrom(
        alice_addr,
        bob_addr,
        vec![token_ids[0]],
        values.clone(),
        vec![].into()
    ))
    .expect_err("should return `ERC1155InvalidArrayLength`");

    assert!(err.reverted_with(Erc1155::ERC1155InvalidArrayLength {
        idsLength: uint!(1_U256),
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
    let contract_alice = Erc1155::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let dave_addr = dave.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let _ = watch!(contract_alice.mintBatch(
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ));
    let _ = watch!(contract_bob.setApprovalForAll(alice_addr, true));

    let err = send!(contract_alice.safeBatchTransferFrom(
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
