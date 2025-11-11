#![cfg(feature = "e2e")]
#![allow(clippy::unreadable_literal)]

use abi::Erc6909;
use alloy::primitives::{uint, Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, Revert};
use eyre::Result;

mod abi;

// ============================================================================
// Integration Tests: ERC-6909 Token
// ============================================================================

#[e2e::test]
async fn mints(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    assert_eq!(U256::ZERO, initial_balance);

    let receipt = receipt!(contract.mint(alice_addr, id, one))?;

    assert!(receipt.emits(Erc6909::Transfer {
        caller: alice_addr,
        sender: Address::ZERO,
        receiver: alice_addr,
        id,
        amount: one,
    }));

    let Erc6909::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    assert_eq!(one, balance);

    Ok(())
}

#[e2e::test]
async fn mints_rejects_invalid_receiver(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let invalid_receiver = Address::ZERO;

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(invalid_receiver, id).call().await?;

    let err = send!(contract.mint(invalid_receiver, id, one))
        .expect_err("should not mint tokens for Address::ZERO");

    assert!(err.reverted_with(Erc6909::ERC6909InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc6909::balanceOfReturn { balance } =
        contract.balanceOf(invalid_receiver, id).call().await?;

    assert_eq!(initial_balance, balance);

    Ok(())
}

#[e2e::test]
async fn burns(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let id = uint!(2_U256);
    let balance = uint!(10_U256);
    let one = uint!(1_U256);

    watch!(contract.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let receipt = receipt!(contract.burn(alice_addr, id, one))?;

    let Erc6909::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    assert!(receipt.emits(Erc6909::Transfer {
        caller: alice_addr,
        sender: alice_addr,
        receiver: Address::ZERO,
        id,
        amount: one,
    }));

    assert_eq!(initial_balance - one, balance);

    Ok(())
}

#[e2e::test]
async fn burn_rejects_insufficient_balance(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let id = uint!(2_U256);
    let balance = uint!(10_U256);
    let balance_plus_one = uint!(11_U256);

    watch!(contract.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let err = send!(contract.burn(alice_addr, id, balance_plus_one))
        .expect_err("should not burn when balance is insufficient");

    assert!(err.reverted_with(Erc6909::ERC6909InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: balance_plus_one,
        id
    }));

    let Erc6909::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    assert_eq!(initial_balance, balance);

    Ok(())
}

#[e2e::test]
async fn burn_rejects_invalid_sender(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let invalid_sender = Address::ZERO;

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let err = send!(contract.burn(invalid_sender, id, one))
        .expect_err("should not burn for invalid sender");

    assert!(err.reverted_with(Erc6909::ERC6909InvalidSender {
        sender: invalid_sender
    }));

    Ok(())
}

#[e2e::test]
async fn transfers(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);
    let balance = uint!(10_U256);

    watch!(contract.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr, id).call().await?;

    assert_eq!(U256::ZERO, initial_bob_balance);

    let receipt = receipt!(contract.transfer(bob_addr, id, one))?;

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr, id).call().await?;

    assert!(receipt.emits(Erc6909::Transfer {
        caller: alice_addr,
        sender: alice_addr,
        receiver: bob_addr,
        id,
        amount: one,
    }));

    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfer_rejects_insufficient_balance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let balance = uint!(10_U256);
    let balance_plus_one = uint!(11_U256);

    watch!(contract.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr, id).call().await?;

    let err = send!(contract.transfer(bob_addr, id, balance_plus_one))
        .expect_err("should not transfer when balance is insufficient");

    assert!(err.reverted_with(Erc6909::ERC6909InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: balance_plus_one,
        id
    }));

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr, id).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);

    Ok(())
}

#[e2e::test]
async fn transfers_from(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract_alice = Erc6909::new(contract_addr, &alice.wallet);
    let contract_bob = Erc6909::new(contract_addr, &bob.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);
    let balance = uint!(10_U256);

    watch!(contract_alice.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    watch!(contract_alice.approve(bob_addr, id, one))?;

    let Erc6909::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    let receipt =
        receipt!(contract_bob.transferFrom(alice_addr, bob_addr, id, one))?;

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    assert!(receipt.emits(Erc6909::Transfer {
        caller: bob_addr,
        sender: alice_addr,
        receiver: bob_addr,
        id,
        amount: one,
    }));

    assert_eq!(initial_alice_balance - one, alice_balance);
    assert_eq!(initial_bob_balance + one, bob_balance);
    assert_eq!(initial_allowance - one, allowance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_reverts_insufficient_balance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract_alice = Erc6909::new(contract_addr, &alice.wallet);
    let contract_bob = Erc6909::new(contract_addr, &bob.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let balance = uint!(10_U256);
    let balance_plus_one = uint!(11_U256);

    watch!(contract_alice.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    watch!(contract_alice.approve(bob_addr, id, balance_plus_one))?;

    let Erc6909::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    let err = send!(contract_bob.transferFrom(
        alice_addr,
        bob_addr,
        id,
        balance_plus_one,
    ))
    .expect_err("should not transfer when balance is insufficient");

    assert!(err.reverted_with(Erc6909::ERC6909InsufficientBalance {
        sender: alice_addr,
        balance,
        needed: balance_plus_one,
        id
    }));

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_rejects_insufficient_allowance(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract_alice = Erc6909::new(contract_addr, &alice.wallet);
    let contract_bob = Erc6909::new(contract_addr, &bob.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);
    let balance = uint!(10_U256);

    watch!(contract_alice.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    assert_eq!(initial_allowance, U256::ZERO);

    let err = send!(contract_bob.transferFrom(alice_addr, bob_addr, id, one))
        .expect_err("should not transfer when insufficient allowance");

    assert!(err.reverted_with(Erc6909::ERC6909InsufficientAllowance {
        spender: bob_addr,
        allowance: U256::ZERO,
        needed: one,
        id
    }));

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn transfer_from_rejects_invalid_receiver(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract_alice = Erc6909::new(contract_addr, &alice.wallet);
    let contract_bob = Erc6909::new(contract_addr, &bob.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let invalid_receiver = Address::ZERO;

    let id = uint!(2_U256);
    let one = uint!(1_U256);
    let balance = uint!(10_U256);

    watch!(contract_alice.mint(alice_addr, id, balance))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    watch!(contract_alice.approve(bob_addr, id, one))?;

    let Erc6909::allowanceReturn { allowance: initial_allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    let err =
        send!(contract_bob.transferFrom(alice_addr, invalid_receiver, id, one))
            .expect_err("should not transfer to Address::ZERO");

    assert!(err.reverted_with(Erc6909::ERC6909InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_allowance, allowance);

    Ok(())
}

#[e2e::test]
async fn approves(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract_alice = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909::allowanceReturn { allowance: initial_alice_to_bob_allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance: initial_bob_to_alice_allowance } =
        contract_alice.allowance(bob_addr, alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    assert_eq!(U256::ZERO, initial_alice_to_bob_allowance);
    assert_eq!(U256::ZERO, initial_bob_to_alice_allowance);

    let receipt = receipt!(contract_alice.approve(bob_addr, id, one))?;

    assert!(receipt.emits(Erc6909::Approval {
        owner: alice_addr,
        spender: bob_addr,
        id,
        amount: one,
    }));

    let Erc6909::allowanceReturn { allowance: alice_to_bob_allowance } =
        contract_alice.allowance(alice_addr, bob_addr, id).call().await?;

    let Erc6909::allowanceReturn { allowance: bob_to_alice_allowance } =
        contract_alice.allowance(bob_addr, alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract_alice.balanceOf(alice_addr, id).call().await?;

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract_alice.balanceOf(bob_addr, id).call().await?;

    assert_eq!(initial_alice_balance, alice_balance);
    assert_eq!(initial_bob_balance, bob_balance);
    assert_eq!(initial_alice_to_bob_allowance + one, alice_to_bob_allowance);
    assert_eq!(initial_bob_to_alice_allowance, bob_to_alice_allowance);

    Ok(())
}

#[e2e::test]
async fn approve_rejects_invalid_spender(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_spender = Address::ZERO;

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909::allowanceReturn { allowance: initial_alice_spender_allowance } =
        contract.allowance(alice_addr, invalid_spender, id).call().await?;

    let Erc6909::allowanceReturn { allowance: initial_spender_alice_allowance } =
        contract.allowance(invalid_spender, alice_addr, id).call().await?;

    assert_eq!(U256::ZERO, initial_alice_spender_allowance);
    assert_eq!(U256::ZERO, initial_spender_alice_allowance);

    let err = send!(contract.approve(invalid_spender, id, one))
        .expect_err("should not approve for Address::ZERO");

    assert!(err.reverted_with(Erc6909::ERC6909InvalidSpender {
        spender: invalid_spender
    }));

    let Erc6909::allowanceReturn { allowance: alice_spender_allowance } =
        contract.allowance(alice_addr, invalid_spender, id).call().await?;

    let Erc6909::allowanceReturn { allowance: spender_alice_allowance } =
        contract.allowance(invalid_spender, alice_addr, id).call().await?;

    assert_eq!(initial_alice_spender_allowance, alice_spender_allowance);
    assert_eq!(initial_spender_alice_allowance, spender_alice_allowance);

    Ok(())
}

#[e2e::test]
async fn sets_operator(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let Erc6909::isOperatorReturn { approved: initial_approved } =
        contract.isOperator(alice_addr, bob_addr).call().await?;

    assert_eq!(false, initial_approved);

    let receipt = receipt!(contract.setOperator(bob_addr, true))?;

    let Erc6909::isOperatorReturn { approved } =
        contract.isOperator(alice_addr, bob_addr).call().await?;

    assert_eq!(true, approved);

    assert!(receipt.emits(Erc6909::OperatorSet {
        owner: alice_addr,
        spender: bob_addr,
        approved: true,
    }));

    Ok(())
}

#[e2e::test]
async fn set_operator_rejects_invalid_spender(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_spender = Address::ZERO;

    let Erc6909::isOperatorReturn { approved: initial_approved } =
        contract.isOperator(alice_addr, invalid_spender).call().await?;

    assert_eq!(false, initial_approved);

    let err = send!(contract.setOperator(invalid_spender, true))
        .expect_err("should not set operator for Address::ZERO");

    assert!(err.reverted_with(Erc6909::ERC6909InvalidSpender {
        spender: invalid_spender
    }));

    let Erc6909::isOperatorReturn { approved } =
        contract.isOperator(alice_addr, invalid_spender).call().await?;

    assert_eq!(false, approved);

    Ok(())
}

#[e2e::test]
async fn supports_interface(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909::new(contract_addr, &alice.wallet);

    let invalid_interface_id: u32 = 0xffffffff;
    let supports_interface = contract
        .supportsInterface(invalid_interface_id.into())
        .call()
        .await?
        .supportsInterface;

    assert!(!supports_interface);

    let erc6909_interface_id: u32 = 0x0f632fb3;
    let supports_interface = contract
        .supportsInterface(erc6909_interface_id.into())
        .call()
        .await?
        .supportsInterface;

    assert!(supports_interface);

    let erc165_interface_id: u32 = 0x01ffc9a7;
    let supports_interface_erc165 = contract
        .supportsInterface(erc165_interface_id.into())
        .call()
        .await?
        .supportsInterface;

    assert!(supports_interface_erc165);

    Ok(())
}
