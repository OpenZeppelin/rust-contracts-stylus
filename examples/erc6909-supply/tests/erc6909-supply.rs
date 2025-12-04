#![cfg(feature = "e2e")]
#![allow(clippy::unreadable_literal)]

use abi::Erc6909TokenSupply;
use alloy::primitives::{uint, Address, U256};
use e2e::{receipt, send, watch, Account, EventExt, Revert, RustPanic};
use eyre::Result;

mod abi;

// ============================================================================
// Integration Tests: ERC-6909 Token Supply Extension
// ============================================================================

#[e2e::test]
async fn mints(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(U256::ZERO, initial_balance);

    let receipt = receipt!(contract.mint(alice_addr, id, one))?;

    assert!(receipt.emits(Erc6909TokenSupply::Transfer {
        caller: alice_addr,
        sender: Address::ZERO,
        receiver: alice_addr,
        id,
        amount: one,
    }));

    let Erc6909TokenSupply::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(initial_balance + one, balance);
    assert_eq!(initial_supply + one, total_supply);

    Ok(())
}

#[e2e::test]
async fn mints_twice(alice: Account, bob: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);
    let ten = uint!(10_U256);

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance_alice } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance_bob } =
        contract.balanceOf(bob_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(U256::ZERO, initial_supply);

    let receipt1 = receipt!(contract.mint(alice_addr, id, one))?;
    let receipt2 = receipt!(contract.mint(bob_addr, id, ten))?;

    assert!(receipt1.emits(Erc6909TokenSupply::Transfer {
        caller: alice_addr,
        sender: Address::ZERO,
        receiver: alice_addr,
        id,
        amount: one,
    }));

    assert!(receipt2.emits(Erc6909TokenSupply::Transfer {
        caller: alice_addr,
        sender: Address::ZERO,
        receiver: bob_addr,
        id,
        amount: ten,
    }));

    let Erc6909TokenSupply::balanceOfReturn { balance: balance_alice } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::balanceOfReturn { balance: balance_bob } =
        contract.balanceOf(bob_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(initial_balance_alice + one, balance_alice);
    assert_eq!(initial_balance_bob + ten, balance_bob);
    assert_eq!(initial_supply + one + ten, total_supply);

    Ok(())
}

#[e2e::test]
async fn mints_rejects_invalid_receiver(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);
    let invalid_receiver = Address::ZERO;

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(invalid_receiver, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply(id).call().await?;

    let err = send!(contract.mint(invalid_receiver, id, one))
        .expect_err("should not mint tokens for Address::ZERO");

    assert!(err.reverted_with(Erc6909TokenSupply::ERC6909InvalidReceiver {
        receiver: invalid_receiver
    }));

    let Erc6909TokenSupply::balanceOfReturn { balance } =
        contract.balanceOf(invalid_receiver, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);

    Ok(())
}

#[e2e::test]
async fn mints_rejects_overflow(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);

    let max_cap = U256::MAX;
    let alice_addr = alice.address();

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    watch!(contract.mint(alice_addr, id, max_cap))?;

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(initial_supply, max_cap);
    assert_eq!(initial_balance, max_cap);

    let err = send!(contract.mint(alice_addr, id, one))
        .expect_err("should not exceed U256::MAX");

    assert!(err.panicked());

    let Erc6909TokenSupply::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);

    Ok(())
}

#[e2e::test]
async fn burns(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();

    let id = uint!(2_U256);
    let balance = uint!(10_U256);
    let one = uint!(1_U256);

    watch!(contract.mint(alice_addr, id, balance))?;

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply(id).call().await?;

    let receipt = receipt!(contract.burn(alice_addr, id, one))?;

    let Erc6909TokenSupply::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply(id).call().await?;

    assert!(receipt.emits(Erc6909TokenSupply::Transfer {
        caller: alice_addr,
        sender: alice_addr,
        receiver: Address::ZERO,
        id,
        amount: one,
    }));

    assert_eq!(initial_balance - one, balance);
    assert_eq!(initial_supply - one, total_supply);

    Ok(())
}

#[e2e::test]
async fn burn_rejects_invalid_sender(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);
    let alice_addr = alice.address();
    let invalid_sender = Address::ZERO;

    let id = uint!(2_U256);
    let one = uint!(1_U256);

    let Erc6909TokenSupply::balanceOfReturn { balance: initial_balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: initial_supply } =
        contract.totalSupply(id).call().await?;

    let err = send!(contract.burn(invalid_sender, id, one))
        .expect_err("should not burn when balance is insufficient");

    assert!(err.reverted_with(Erc6909TokenSupply::ERC6909InvalidSender {
        sender: invalid_sender
    }));

    let Erc6909TokenSupply::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, id).call().await?;

    let Erc6909TokenSupply::totalSupplyReturn { totalSupply: total_supply } =
        contract.totalSupply(id).call().await?;

    assert_eq!(initial_balance, balance);
    assert_eq!(initial_supply, total_supply);

    Ok(())
}

#[e2e::test]
async fn supports_interface(alice: Account) -> Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;

    let contract = Erc6909TokenSupply::new(contract_addr, &alice.wallet);

    let invalid_interface_id: u32 = 0xffffffff;
    let supports_interface = contract
        .supportsInterface(invalid_interface_id.into())
        .call()
        .await?
        .supportsInterface;

    assert!(!supports_interface);

    let erc6909_interface_id: u32 = 0xbd85b039;
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
