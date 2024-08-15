#![cfg(feature = "e2e")]

use abi::{Ownable, Ownable::OwnershipTransferred};
use alloy::{
    network::ReceiptResponse,
    primitives::Address,
    providers::Provider,
    sol,
    sol_types::{SolConstructor, SolError, SolEvent},
};
use e2e::{deploy, receipt, send, Account, EventExt, ReceiptExt, Revert};
use eyre::{ContextCompat, Result};

use crate::OwnableExample::constructorCall;

mod abi;

sol!("src/constructor.sol");

fn constructor(owner: Address) -> Option<constructorCall> {
    Some(constructorCall { initialOwner: owner })
}

// ============================================================================
// Integration Tests: Ownable
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let receipt = deploy(&alice, constructor(alice_addr)).await?;
    let contract = Ownable::new(receipt.address()?, &alice.wallet);

    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: Address::ZERO,
        newOwner: alice_addr,
    }));

    let Ownable::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);
    Ok(())
}

#[e2e::test]
async fn rejects_zero_address_initial_owner(alice: Account) -> Result<()> {
    let err = deploy(&alice, constructor(Address::ZERO))
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    assert!(err
        .reverted_with(Ownable::OwnableInvalidOwner { owner: Address::ZERO }));
    Ok(())
}

#[e2e::test]
async fn transfers_ownership(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr =
        deploy(&alice, constructor(alice_addr)).await?.address()?;
    let contract = Ownable::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.transferOwnership(bob_addr))?;
    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: alice_addr,
        newOwner: bob_addr,
    }));

    let Ownable::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, bob_addr);

    Ok(())
}

#[e2e::test]
async fn prevents_non_owners_from_transferring(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr =
        deploy(&alice, constructor(bob_addr)).await?.address()?;
    let contract = Ownable::new(contract_addr, &alice.wallet);

    let err = send!(contract.transferOwnership(bob_addr))
        .expect_err("should not transfer when not owned");
    err.reverted_with(Ownable::OwnableUnauthorizedAccount {
        account: alice_addr,
    });

    Ok(())
}

#[e2e::test]
async fn guards_against_stuck_state(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr =
        deploy(&alice, constructor(alice_addr)).await?.address()?;
    let contract = Ownable::new(contract_addr, &alice.wallet);

    let err = send!(contract.transferOwnership(Address::ZERO))
        .expect_err("should not transfer to Address::ZERO");
    err.reverted_with(Ownable::OwnableInvalidOwner { owner: Address::ZERO });

    let Ownable::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);

    Ok(())
}

#[e2e::test]
async fn loses_ownership_after_renouncement(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr =
        deploy(&alice, constructor(alice_addr)).await?.address()?;
    let contract = Ownable::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.renounceOwnership())?;
    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: alice_addr,
        newOwner: Address::ZERO,
    }));

    let Ownable::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, Address::ZERO);

    Ok(())
}

#[e2e::test]
async fn prevents_non_owners_from_renouncement(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr =
        deploy(&alice, constructor(alice_addr)).await?.address()?;
    let contract = Ownable::new(contract_addr, &bob.wallet);

    let err = send!(contract.renounceOwnership())
        .expect_err("should prevent non-owner from renouncing");
    err.reverted_with(Ownable::OwnableUnauthorizedAccount {
        account: bob_addr,
    });

    let Ownable::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);

    Ok(())
}
