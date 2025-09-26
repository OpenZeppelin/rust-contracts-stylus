#![cfg(feature = "e2e")]

use abi::{
    Ownable2Step,
    Ownable2Step::{OwnershipTransferStarted, OwnershipTransferred},
};
use alloy::primitives::Address;
use e2e::{
    constructor, receipt, send, Account, Constructor,
    ContractInitializationError, EventExt, Revert,
};
use eyre::Result;

mod abi;

fn ctr(owner: Address) -> Constructor {
    constructor!(owner)
}

// ============================================================================
// Integration Tests: Ownable2Step
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let receipt =
        alice.as_deployer().with_constructor(ctr(alice_addr)).deploy().await?;
    let contract = Ownable2Step::new(receipt.contract_address, &alice.wallet);

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
async fn construct_reverts_when_owner_is_zero_address(
    alice: Account,
) -> Result<()> {
    let err = alice
        .as_deployer()
        .with_constructor(ctr(Address::ZERO))
        .deploy()
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    // TODO: assert the actual `OwnableInvalidOwner` error was returned once
    // `StylusDeployer` is able to return the exact revert reason from
    // constructors. assert!(err.
    // reverted_with(Ownable2Step::OwnableInvalidOwner {     owner:
    // Address::ZERO }));

    assert!(err.downcast_ref::<ContractInitializationError>().is_some());

    Ok(())
}

#[e2e::test]
async fn transfer_ownership_initiates_transfer(
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
        .contract_address;
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
        .contract_address;
    let contract = Ownable2Step::new(contract_addr, &alice.wallet);

    let err = send!(contract.transferOwnership(bob_addr))
        .expect_err("should not transfer when not owned");
    err.reverted_with(Ownable2Step::OwnableUnauthorizedAccount {
        account: alice_addr,
    });

    Ok(())
}

#[e2e::test]
async fn accept_ownership(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;
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
async fn transfer_ownership_cancel_transfer(
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
        .contract_address;
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
async fn overwrite_previous_transfer_ownership(
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
        .contract_address;

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
        .contract_address;
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
async fn renounce_ownership(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;
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
        .contract_address;
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
