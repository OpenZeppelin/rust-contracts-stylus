#![cfg(feature = "e2e")]
#![allow(clippy::unreadable_literal)]

use abi::Erc1155Supply;
use alloy::primitives::{aliases::B32, uint, Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, RustPanic};
use mock::{receiver, receiver::ERC1155ReceiverMock};

mod abi;
mod mock;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(U256::from).collect()
}

fn random_values(size: usize) -> Vec<U256> {
    (1..=size).map(U256::from).collect()
}

// ============================================================================
// Integration Tests: ERC-1155 Supply Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let token_id = random_token_ids(1)[0];

    let total_supply = contract.totalSupply_0(token_id).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;
    let token_exists = contract.exists(token_id).call().await?._0;

    assert_eq!(U256::ZERO, total_supply);
    assert_eq!(U256::ZERO, total_supply_all);
    assert!(!token_exists);

    Ok(())
}

#[e2e::test]
async fn mint(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let receipt =
        receipt!(contract.mint(alice_addr, token_id, value, vec![].into()))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: Address::ZERO,
        to: alice_addr,
        id: token_id,
        value,
    }));

    let balance =
        contract.balanceOf(alice_addr, token_id).call().await?.balance;
    let total_supply = contract.totalSupply_0(token_id).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;
    let token_exists = contract.exists(token_id).call().await?._0;

    assert_eq!(value, balance);
    assert_eq!(value, total_supply);
    assert_eq!(value, total_supply_all);
    assert!(token_exists);

    Ok(())
}

#[e2e::test]
async fn mint_to_receiver_contract(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let receiver_addr =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    let initial_receiver_balance =
        contract.balanceOf(receiver_addr, token_id).call().await?.balance;

    let receipt =
        receipt!(contract.mint(receiver_addr, token_id, value, vec![].into()))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
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

    let receiver_balance =
        contract.balanceOf(receiver_addr, token_id).call().await?.balance;
    let total_supply = contract.totalSupply_0(token_id).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;
    let token_exists = contract.exists(token_id).call().await?._0;

    assert_eq!(initial_receiver_balance + value, receiver_balance);
    assert_eq!(value, total_supply);
    assert_eq!(value, total_supply_all);
    assert!(token_exists);

    Ok(())
}

#[e2e::test]
async fn mint_batch(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let accounts = vec![alice_addr, bob_addr];

    for &account in &accounts {
        let receipt = receipt!(contract.mintBatch(
            account,
            token_ids.clone(),
            values.clone(),
            vec![].into()
        ))?;

        assert!(receipt.emits(Erc1155Supply::TransferBatch {
            operator: alice_addr,
            from: Address::ZERO,
            to: account,
            ids: token_ids.clone(),
            values: values.clone()
        }));

        let balances = contract
            .balanceOfBatch(vec![account, account], token_ids.clone())
            .call()
            .await?
            .balances;

        assert_eq!(values, balances);
    }

    let accounts_len = U256::from(accounts.len());

    for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
        let token_exists = contract.exists(token_id).call().await?._0;
        let total_supply = contract.totalSupply_0(token_id).call().await?._0;

        assert_eq!(value * accounts_len, total_supply);
        assert!(token_exists);
    }

    let total_supply_all = contract.totalSupply_1().call().await?._0;
    assert_eq!(values.iter().sum::<U256>() * accounts_len, total_supply_all);

    Ok(())
}

#[e2e::test]
async fn mint_batch_transfer_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let receiver_addr =
        receiver::deploy(&alice.wallet, ERC1155ReceiverMock::RevertType::None)
            .await?;

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);
    let values = random_values(2);

    let initial_receiver_balances = contract
        .balanceOfBatch(vec![receiver_addr, receiver_addr], token_ids.clone())
        .call()
        .await?
        .balances;

    let receipt = receipt!(contract.mintBatch(
        receiver_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
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

    let receiver_balances = contract
        .balanceOfBatch(vec![receiver_addr, receiver_addr], token_ids.clone())
        .call()
        .await?
        .balances;

    for (idx, (&token_id, &value)) in
        token_ids.iter().zip(values.iter()).enumerate()
    {
        let token_exists = contract.exists(token_id).call().await?._0;
        let total_supply = contract.totalSupply_0(token_id).call().await?._0;

        assert_eq!(
            initial_receiver_balances[idx] + value,
            receiver_balances[idx]
        );
        assert_eq!(value, total_supply);
        assert!(token_exists);
    }

    let total_supply_all = contract.totalSupply_1().call().await?._0;
    assert_eq!(values.iter().sum::<U256>(), total_supply_all);

    Ok(())
}

#[e2e::test]
async fn mint_panics_on_total_supply_overflow(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let two = uint!(2_U256);
    let three = uint!(3_U256);

    watch!(contract.mint(
        alice_addr,
        token_id,
        U256::MAX / two,
        vec![].into()
    ))?;
    watch!(contract.mint(bob_addr, token_id, U256::MAX / two, vec![].into()))?;

    let err = send!(contract.mint(alice_addr, token_id, three, vec![].into()))
        .expect_err("should panic due to total_supply overflow");

    assert!(err.panicked());

    Ok(())
}

#[e2e::test]
async fn mint_panics_on_total_supply_all_overflow(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);

    watch!(contract.mint(alice_addr, token_ids[0], U256::MAX, vec![].into()))?;

    let err = send!(contract.mint(
        alice_addr,
        token_ids[1],
        U256::ONE,
        vec![].into()
    ))
    .expect_err("should panic due to total_supply_all overflow");

    assert!(err.panicked());

    Ok(())
}

#[e2e::test]
async fn burn(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(alice_addr, token_id, value, vec![].into()))?;

    let receipt = receipt!(contract.burn(alice_addr, token_id, value))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: Address::ZERO,
        id: token_id,
        value,
    }));

    let token_exists = contract.exists(token_id).call().await?._0;
    let balance =
        contract.balanceOf(alice_addr, token_id).call().await?.balance;
    let total_supply = contract.totalSupply_0(token_id).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;

    assert_eq!(U256::ZERO, balance);
    assert_eq!(U256::ZERO, total_supply);
    assert_eq!(U256::ZERO, total_supply_all);
    assert!(!token_exists);

    Ok(())
}

#[e2e::test]
async fn burn_with_approval(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155Supply::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(bob_addr, token_id, value, vec![].into()))?;

    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let receipt = receipt!(contract.burn(bob_addr, token_id, value))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: bob_addr,
        to: Address::ZERO,
        id: token_id,
        value,
    }));

    let token_exists = contract.exists(token_id).call().await?._0;
    let balance = contract.balanceOf(bob_addr, token_id).call().await?.balance;
    let total_supply = contract.totalSupply_0(token_id).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;

    assert_eq!(U256::ZERO, balance);
    assert_eq!(U256::ZERO, total_supply);
    assert_eq!(U256::ZERO, total_supply_all);
    assert!(!token_exists);

    Ok(())
}

#[e2e::test]
async fn burn_batch(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(4);
    let values = random_values(4);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    let receipt = receipt!(contract.burnBatch(
        alice_addr,
        token_ids.clone(),
        values.clone()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
        operator: alice_addr,
        from: alice_addr,
        to: Address::ZERO,
        ids: token_ids.clone(),
        values,
    }));

    for token_id in token_ids {
        let balance =
            contract.balanceOf(alice_addr, token_id).call().await?.balance;
        let token_exists = contract.exists(token_id).call().await?._0;
        let total_supply = contract.totalSupply_0(token_id).call().await?._0;

        assert_eq!(U256::ZERO, balance);
        assert_eq!(U256::ZERO, total_supply);
        assert!(!token_exists);
    }

    let total_supply_all = contract.totalSupply_1().call().await?._0;
    assert_eq!(U256::ZERO, total_supply_all);

    Ok(())
}

#[e2e::test]
async fn burn_batch_with_approval(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155Supply::new(contract_addr, &bob.wallet);

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

    watch!(contract_bob.setApprovalForAll(alice_addr, true))?;

    let receipt = receipt!(contract.burnBatch(
        bob_addr,
        token_ids.clone(),
        values.clone()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
        operator: alice_addr,
        from: bob_addr,
        to: Address::ZERO,
        ids: token_ids.clone(),
        values,
    }));

    for token_id in token_ids {
        let balance =
            contract.balanceOf(bob_addr, token_id).call().await?.balance;
        let token_exists = contract.exists(token_id).call().await?._0;
        let total_supply = contract.totalSupply_0(token_id).call().await?._0;

        assert_eq!(U256::ZERO, balance);
        assert_eq!(U256::ZERO, total_supply);
        assert!(!token_exists);
    }

    let total_supply_all = contract.totalSupply_1().call().await?._0;
    assert_eq!(U256::ZERO, total_supply_all);

    Ok(())
}

#[e2e::test]
async fn supply_unaffected_by_safe_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];

    watch!(contract.mint(alice_addr, token_id, value, vec![].into()))?;

    // assert balances as expected after mint
    let alice_balance =
        contract.balanceOf(alice_addr, token_id).call().await?.balance;
    let bob_balance =
        contract.balanceOf(bob_addr, token_id).call().await?.balance;

    assert_eq!(value, alice_balance);
    assert_eq!(U256::ZERO, bob_balance);

    // total supplies (all) logic has been checked in other tests, assume valid
    let initial_total_supply =
        contract.totalSupply_0(token_id).call().await?._0;
    let initial_total_supply_all = contract.totalSupply_1().call().await?._0;

    let receipt = receipt!(contract.safeTransferFrom(
        alice_addr,
        bob_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: bob_addr,
        id: token_id,
        value,
    }));

    // assert balances updated as expected
    let alice_balance =
        contract.balanceOf(alice_addr, token_id).call().await?.balance;
    let bob_balance =
        contract.balanceOf(bob_addr, token_id).call().await?.balance;

    assert_eq!(U256::ZERO, alice_balance);
    assert_eq!(value, bob_balance);

    // assert supply-related data remains unchanged
    let total_supply = contract.totalSupply_0(token_id).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;
    let token_exists = contract.exists(token_id).call().await?._0;

    assert_eq!(initial_total_supply, total_supply);
    assert_eq!(initial_total_supply_all, total_supply_all);
    assert!(token_exists);

    Ok(())
}

#[e2e::test]
async fn supply_unaffected_by_safe_transfer_from_batch(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_ids = random_token_ids(4);
    let values = random_values(4);

    watch!(contract.mintBatch(
        alice_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    // assert balances as expected after mint
    for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
        let alice_balance =
            contract.balanceOf(alice_addr, token_id).call().await?.balance;
        let bob_balance =
            contract.balanceOf(bob_addr, token_id).call().await?.balance;

        assert_eq!(value, alice_balance);
        assert_eq!(U256::ZERO, bob_balance);
    }

    // total supplies (all) logic has been checked in other tests, assume valid
    let mut initial_total_supplies: Vec<U256> = vec![];
    for &token_id in &token_ids {
        let supply = contract.totalSupply_0(token_id).call().await?._0;
        initial_total_supplies.push(supply);
    }
    let initial_total_supply_all = contract.totalSupply_1().call().await?._0;

    let receipt = receipt!(contract.safeBatchTransferFrom(
        alice_addr,
        bob_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
        operator: alice_addr,
        from: alice_addr,
        to: bob_addr,
        ids: token_ids.clone(),
        values: values.clone(),
    }));

    // assert balances updated as expected
    for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
        let alice_balance =
            contract.balanceOf(alice_addr, token_id).call().await?.balance;
        let bob_balance =
            contract.balanceOf(bob_addr, token_id).call().await?.balance;

        assert_eq!(U256::ZERO, alice_balance);
        assert_eq!(value, bob_balance);
    }

    // assert supply-related data remains unchanged
    for (&token_id, &initial_total_supply) in
        token_ids.iter().zip(initial_total_supplies.iter())
    {
        let total_supply = contract.totalSupply_0(token_id).call().await?._0;
        let token_exists = contract.exists(token_id).call().await?._0;

        assert_eq!(initial_total_supply, total_supply);
        assert!(token_exists);
    }

    let total_supply_all = contract.totalSupply_1().call().await?._0;
    assert_eq!(initial_total_supply_all, total_supply_all);

    Ok(())
}

// =====================================================================
// Integration Tests: Happy Paths of Re-exported functions from ERC-1155
// =====================================================================

#[e2e::test]
async fn balance_of_zero_balance(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);
    let token_ids = random_token_ids(1);

    let Erc1155Supply::balanceOfReturn { balance } =
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
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);
    let accounts =
        vec![alice.address(), bob.address(), dave.address(), charlie.address()];
    let token_ids = random_token_ids(4);

    let Erc1155Supply::balanceOfBatchReturn { balances } =
        contract.balanceOfBatch(accounts, token_ids).call().await?;
    assert_eq!(vec![U256::ZERO, U256::ZERO, U256::ZERO, U256::ZERO], balances);

    Ok(())
}

#[e2e::test]
async fn set_approval_for_all(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let approved_value = true;
    let receipt =
        receipt!(contract.setApprovalForAll(bob_addr, approved_value))?;

    assert!(receipt.emits(Erc1155Supply::ApprovalForAll {
        account: alice_addr,
        operator: bob_addr,
        approved: approved_value,
    }));

    let Erc1155Supply::isApprovedForAllReturn { approved } =
        contract.isApprovedForAll(alice_addr, bob_addr).call().await?;
    assert_eq!(approved_value, approved);

    let approved_value = false;
    let receipt =
        receipt!(contract.setApprovalForAll(bob_addr, approved_value))?;

    assert!(receipt.emits(Erc1155Supply::ApprovalForAll {
        account: alice_addr,
        operator: bob_addr,
        approved: approved_value,
    }));

    let Erc1155Supply::isApprovedForAllReturn { approved } =
        contract.isApprovedForAll(alice_addr, bob_addr).call().await?;
    assert_eq!(approved_value, approved);

    Ok(())
}

#[e2e::test]
async fn is_approved_for_all_zero_address(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let invalid_operator = Address::ZERO;

    let Erc1155Supply::isApprovedForAllReturn { approved } = contract
        .isApprovedForAll(alice.address(), invalid_operator)
        .call()
        .await?;

    assert_eq!(false, approved);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

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

    let Erc1155Supply::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    let Erc1155Supply::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr, token_id).call().await?;

    let receipt = receipt!(contract.safeTransferFrom(
        alice_addr,
        bob_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: bob_addr,
        id: token_id,
        value
    }));

    let Erc1155Supply::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance - value, alice_balance);

    let Erc1155Supply::balanceOfReturn { balance: bob_balance } =
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
    let contract_alice = Erc1155Supply::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155Supply::new(contract_addr, &bob.wallet);

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

    let Erc1155Supply::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, token_id).call().await?;
    let Erc1155Supply::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, token_id).call().await?;

    let receipt = receipt!(contract_alice.safeTransferFrom(
        bob_addr,
        alice_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: bob_addr,
        to: alice_addr,
        id: token_id,
        value
    }));

    let Erc1155Supply::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance + value, alice_balance);

    let Erc1155Supply::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, token_id).call().await?;
    assert_eq!(initial_bob_balance - value, bob_balance);

    Ok(())
}

#[e2e::test]
async fn safe_transfer_to_receiver_contract(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

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

    let Erc1155Supply::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    let Erc1155Supply::balanceOfReturn { balance: initial_receiver_balance } =
        contract.balanceOf(receiver_addr, token_id).call().await?;

    let receipt = receipt!(contract.safeTransferFrom(
        alice_addr,
        receiver_addr,
        token_id,
        value,
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
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

    let Erc1155Supply::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance - value, alice_balance);

    let Erc1155Supply::balanceOfReturn { balance: receiver_balance } =
        contract.balanceOf(receiver_addr, token_id).call().await?;
    assert_eq!(initial_receiver_balance + value, receiver_balance);

    Ok(())
}

#[e2e::test]
async fn safe_batch_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155Supply::new(contract_addr, &alice.wallet);

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

    let Erc1155Supply::balanceOfBatchReturn {
        balances: initial_alice_balances,
    } = contract_alice
        .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
        .call()
        .await?;

    let Erc1155Supply::balanceOfBatchReturn { balances: initial_bob_balances } =
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

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
        operator: alice_addr,
        from: alice_addr,
        to: bob_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    let Erc1155Supply::balanceOfBatchReturn { balances: alice_balances } =
        contract_alice
            .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155Supply::balanceOfBatchReturn { balances: bob_balances } =
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
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

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

    let Erc1155Supply::balanceOfBatchReturn {
        balances: initial_alice_balances,
    } = contract
        .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
        .call()
        .await?;

    let Erc1155Supply::balanceOfBatchReturn {
        balances: initial_receiver_balances,
    } = contract
        .balanceOfBatch(vec![receiver_addr, receiver_addr], token_ids.clone())
        .call()
        .await?;

    let receipt = receipt!(contract.safeBatchTransferFrom(
        alice_addr,
        receiver_addr,
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
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

    let Erc1155Supply::balanceOfBatchReturn { balances: alice_balances } =
        contract
            .balanceOfBatch(vec![alice_addr, alice_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155Supply::balanceOfBatchReturn { balances: receiver_balances } =
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
async fn safe_batch_transfer_from_with_approval(
    alice: Account,
    bob: Account,
    dave: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract_alice = Erc1155Supply::new(contract_addr, &alice.wallet);
    let contract_bob = Erc1155Supply::new(contract_addr, &bob.wallet);

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

    let Erc1155Supply::balanceOfBatchReturn { balances: initial_dave_balances } =
        contract_alice
            .balanceOfBatch(vec![dave_addr, dave_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155Supply::balanceOfBatchReturn { balances: initial_bob_balances } =
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

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
        operator: alice_addr,
        from: bob_addr,
        to: dave_addr,
        ids: token_ids.clone(),
        values: values.clone()
    }));

    let Erc1155Supply::balanceOfBatchReturn { balances: bob_balances } =
        contract_alice
            .balanceOfBatch(vec![bob_addr, bob_addr], token_ids.clone())
            .call()
            .await?;

    let Erc1155Supply::balanceOfBatchReturn { balances: dave_balances } =
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

// ============================================================================
// Integration Tests: ERC-165 Support Interface
// ============================================================================

#[e2e::test]
async fn supports_interface(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);
    let invalid_interface_id: B32 = 0xffffffff_u32.into();
    let supports_interface =
        contract.supportsInterface(invalid_interface_id).call().await?._0;

    assert!(!supports_interface);

    let erc1155_interface_id: B32 = 0xd9b67a26_u32.into();
    let supports_interface =
        contract.supportsInterface(erc1155_interface_id).call().await?._0;

    assert!(supports_interface);

    let erc165_interface_id: B32 = 0x01ffc9a7_u32.into();
    let supports_interface =
        contract.supportsInterface(erc165_interface_id).call().await?._0;

    assert!(supports_interface);

    Ok(())
}
