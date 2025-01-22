#![cfg(feature = "e2e")]

use abi::{
    Ownable2Step,
    Ownable2Step::{OwnershipTransferStarted, OwnershipTransferred},
};
use alloy::{primitives::Address, sol};
use e2e::{receipt, send, Account, EventExt, ReceiptExt, Revert};
use eyre::Result;

use crate::Ownable2StepExample::constructorCall;

mod abi;

sol!("src/constructor.sol");

fn ctr(owner: Address) -> constructorCall {
    constructorCall { initialOwner: owner }
}

// ============================================================================
// Integration Tests: Ownable2Step
// ============================================================================

#[e2e::test]
async fn constructor_success(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let receipt =
        alice.as_deployer().with_constructor(ctr(alice_addr)).deploy().await?;
    let contract = Ownable2Step::new(receipt.address()?, &alice.wallet);

    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: Address::ZERO,
        newOwner: alice_addr,
    }));

    let Ownable2Step::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);

    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, Address::ZERO);

    Ok(())
}

#[e2e::test]
async fn constructor_reverts_when_owner_zero(alice: Account) -> Result<()> {
    let err = alice
        .as_deployer()
        .with_constructor(ctr(Address::ZERO))
        .deploy()
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    assert!(err.reverted_with(Ownable2Step::OwnableInvalidOwner {
        owner: Address::ZERO
    }));

    Ok(())
}

#[e2e::test]
async fn transfer_ownership_success(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.transferOwnership(bob_addr))?;
    assert!(receipt.emits(OwnershipTransferStarted {
        previousOwner: alice_addr,
        newOwner: bob_addr,
    }));

    // Current owner is still Alice
    let Ownable2Step::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);

    // Pending owner is Bob
    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, bob_addr);

    Ok(())
}

#[e2e::test]
async fn transfer_ownership_reverts_when_not_owner(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(bob_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);

    let err = send!(contract.transferOwnership(bob_addr))
        .expect_err("should not transfer when not owned");
    err.reverted_with(Ownable2Step::OwnableUnauthorizedAccount {
        account: alice_addr,
    });

    Ok(())
}

#[e2e::test]
async fn accept_ownership_success(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);
    receipt!(contract.transferOwnership(bob_addr))?;

    // Connect as Bob and accept ownership
    let contract = Ownable2Step::new(contract_addr, &bob.wallet);
    let receipt = receipt!(contract.acceptOwnership())?;
    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: alice_addr,
        newOwner: bob_addr,
    }));

    let Ownable2Step::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, bob_addr);

    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, Address::ZERO);

    Ok(())
}

#[e2e::test]
async fn transfer_ownership_success_with_cancel(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);
    receipt!(contract.transferOwnership(bob_addr))?;

    let receipt = receipt!(contract.transferOwnership(Address::ZERO))?;
    assert!(receipt.emits(OwnershipTransferStarted {
        previousOwner: alice_addr,
        newOwner: Address::ZERO,
    }));

    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, Address::ZERO);

    Ok(())
}

#[e2e::test]
async fn transfer_ownership_success_with_overwrite(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let charlie_addr = charlie.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;

    let contract = Ownable2Step::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.transferOwnership(bob_addr))?;
    assert!(receipt.emits(OwnershipTransferStarted {
        previousOwner: alice_addr,
        newOwner: bob_addr,
    }));

    let receipt = receipt!(contract.transferOwnership(charlie_addr))?;
    assert!(receipt.emits(OwnershipTransferStarted {
        previousOwner: alice_addr,
        newOwner: charlie_addr,
    }));

    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, charlie_addr);

    // Connect as Bob and try to accept ownership
    let contract = Ownable2Step::new(contract_addr, &bob.wallet);
    let err = send!(contract.acceptOwnership())
        .expect_err("should not accept when not pending owner");

    err.reverted_with(Ownable2Step::OwnableUnauthorizedAccount {
        account: bob_addr,
    });

    // Connect as Charlie and accept ownership
    let contract = Ownable2Step::new(contract_addr, &charlie.wallet);
    let receipt = receipt!(contract.acceptOwnership())?;
    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: alice_addr,
        newOwner: charlie_addr,
    }));

    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, Address::ZERO);

    let Ownable2Step::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, charlie_addr);

    Ok(())
}

#[e2e::test]
async fn accept_ownership_reverts_when_not_pending_owner(
    alice: Account,
    bob: Account,
    charlie: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let charlie_addr = charlie.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);
    receipt!(contract.transferOwnership(bob_addr))?;

    // Connect as Charlie and attempt to accept ownership
    let contract = Ownable2Step::new(contract_addr, &charlie.wallet);
    let err = send!(contract.acceptOwnership())
        .expect_err("should not accept when not pending owner");
    err.reverted_with(Ownable2Step::OwnableUnauthorizedAccount {
        account: charlie_addr,
    });

    Ok(())
}

#[e2e::test]
async fn renounce_ownership_success(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);

    let receipt = receipt!(contract.renounceOwnership())?;
    assert!(receipt.emits(OwnershipTransferred {
        previousOwner: alice_addr,
        newOwner: Address::ZERO,
    }));

    let Ownable2Step::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, Address::ZERO);

    // Pending owner is set to zero
    let Ownable2Step::pendingOwnerReturn { pendingOwner } =
        contract.pendingOwner().call().await?;
    assert_eq!(pendingOwner, Address::ZERO);

    Ok(())
}

#[e2e::test]
async fn renounce_ownership_reverts_when_not_owner(
    alice: Account,
    bob: Account,
) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .address()?;
    let contract = Ownable2Step::new(contract_addr, &bob.wallet);

    let err = send!(contract.renounceOwnership())
        .expect_err("should prevent non-owner from renouncing");
    err.reverted_with(Ownable2Step::OwnableUnauthorizedAccount {
        account: bob_addr,
    });

    let Ownable2Step::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);

    Ok(())
}
