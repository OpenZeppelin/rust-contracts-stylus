#![cfg(feature = "e2e")]

use abi::Erc1155Supply;
use alloy::{
    primitives::{Address, U256},
    rpc::types::TransactionReceipt,
};
use e2e::{
    receipt, send, watch, Account, EventExt, Panic, PanicCode, ReceiptExt,
};

mod abi;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(U256::from).collect()
}

fn random_values(size: usize) -> Vec<U256> {
    (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
}

async fn deploy_and_mint(
    receiver: &Account,
    size: usize,
) -> eyre::Result<(Address, Vec<U256>, Vec<U256>, TransactionReceipt)> {
    let contract_addr = receiver.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &receiver.wallet);

    let token_ids = random_token_ids(size);
    let values = random_values(size);

    let receipt = receipt!(contract.mintBatch(
        receiver.address(),
        token_ids.clone(),
        values.clone(),
        vec![].into()
    ))?;

    Ok((contract_addr, token_ids, values, receipt))
}

// ============================================================================
// Integration Tests: ERC-1155 Supply Extension
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
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
async fn after_mint_single(alice: Account) -> eyre::Result<()> {
    let (contract_addr, token_ids, values, receipt) =
        deploy_and_mint(&alice, 1).await?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: Address::ZERO,
        to: alice_addr,
        id: token_ids[0],
        value: values[0],
    }));

    let balance =
        contract.balanceOf(alice_addr, token_ids[0]).call().await?.balance;
    let total_supply = contract.totalSupply_0(token_ids[0]).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;
    let token_exists = contract.exists(token_ids[0]).call().await?._0;

    assert_eq!(values[0], balance);
    assert_eq!(values[0], total_supply);
    assert_eq!(values[0], total_supply_all);
    assert!(token_exists);

    Ok(())
}

#[e2e::test]
async fn after_mint_batch(alice: Account) -> eyre::Result<()> {
    let (contract_addr, token_ids, values, receipt) =
        deploy_and_mint(&alice, 4).await?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    assert!(receipt.emits(Erc1155Supply::TransferBatch {
        operator: alice_addr,
        from: Address::ZERO,
        to: alice_addr,
        ids: token_ids.clone(),
        values: values.clone(),
    }));

    for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
        let token_exists = contract.exists(token_id).call().await?._0;
        let total_supply = contract.totalSupply_0(token_id).call().await?._0;
        let balance =
            contract.balanceOf(alice_addr, token_id).call().await?.balance;

        assert_eq!(value, balance);
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
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let two = U256::from(2);
    let three = U256::from(3);

    let _ = watch!(contract.mint(
        alice_addr,
        token_id,
        U256::MAX / two,
        vec![].into()
    ));
    let _ = watch!(contract.mint(
        bob_addr,
        token_id,
        U256::MAX / two,
        vec![].into()
    ));

    let err = send!(contract.mint(alice_addr, token_id, three, vec![].into()))
        .expect_err("should panic due to total_supply overflow");

    assert!(err.panicked_with(PanicCode::ArithmeticOverflow));

    Ok(())
}

#[e2e::test]
async fn mint_panics_on_total_supply_all_overflow(
    alice: Account,
) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.address()?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_ids = random_token_ids(2);

    let _ = watch!(contract.mint(
        alice_addr,
        token_ids[0],
        U256::MAX,
        vec![].into()
    ));

    let err = send!(contract.mint(
        alice_addr,
        token_ids[1],
        U256::from(1),
        vec![].into()
    ))
    .expect_err("should panic due to total_supply_all overflow");

    assert!(err.panicked_with(PanicCode::ArithmeticOverflow));

    Ok(())
}

#[e2e::test]
async fn after_burn_single(alice: Account) -> eyre::Result<()> {
    let (contract_addr, token_ids, values, _) =
        deploy_and_mint(&alice, 1).await?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

    let receipt = receipt!(contract.burn(alice_addr, token_ids[0], values[0]))?;

    assert!(receipt.emits(Erc1155Supply::TransferSingle {
        operator: alice_addr,
        from: alice_addr,
        to: Address::ZERO,
        id: token_ids[0],
        value: values[0],
    }));

    let token_exists = contract.exists(token_ids[0]).call().await?._0;
    let balance =
        contract.balanceOf(alice_addr, token_ids[0]).call().await?.balance;
    let total_supply = contract.totalSupply_0(token_ids[0]).call().await?._0;
    let total_supply_all = contract.totalSupply_1().call().await?._0;

    assert_eq!(U256::ZERO, balance);
    assert_eq!(U256::ZERO, total_supply);
    assert_eq!(U256::ZERO, total_supply_all);
    assert!(!token_exists);

    Ok(())
}

#[e2e::test]
async fn after_burn_batch(alice: Account) -> eyre::Result<()> {
    let (contract_addr, token_ids, values, _) =
        deploy_and_mint(&alice, 4).await?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();

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

    for &token_id in &token_ids {
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
async fn supply_unaffected_by_safe_transfer_from(
    alice: Account,
    bob: Account,
) -> eyre::Result<()> {
    let (contract_addr, token_ids, values, _) =
        deploy_and_mint(&alice, 1).await?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = token_ids[0];
    let value = values[0];

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
    let (contract_addr, token_ids, values, _) =
        deploy_and_mint(&alice, 4).await?;
    let contract = Erc1155Supply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

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
