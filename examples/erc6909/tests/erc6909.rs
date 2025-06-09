#![cfg(feature = "e2e")]

use abi::Erc6909;
use alloy::primitives::{uint, Address, U256};
use e2e::{
    constructor, receipt, send, watch, Account, Constructor,
    ContractInitializationError, EventExt, Panic, PanicCode, Revert,
};
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

    let Erc6909::balanceOfReturn { balance: balance } =
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

    let Erc6909::balanceOfReturn { balance: balance } =
        contract.balanceOf(invalid_receiver, id).call().await?;

    assert_eq!(initial_balance, balance);

    Ok(())
}
