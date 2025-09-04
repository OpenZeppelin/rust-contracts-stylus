#![cfg(feature = "e2e")]

use abi::{Ownable, Ownable::OwnershipTransferred};
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
// Integration Tests: Ownable
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let receipt =
        alice.as_deployer().with_constructor(ctr(alice_addr)).deploy().await?;
    let contract = Ownable::new(receipt.contract_address, &alice.wallet);

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
    let err = alice
        .as_deployer()
        .with_constructor(ctr(Address::ZERO))
        .deploy()
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    // TODO: assert the actual `OwnableInvalidOwner` error was returned once
    // `StylusDeployer` is able to return the exact revert reason from
    // constructors.
    // assert!(err.reverted_with(Ownable::OwnableInvalidOwner { owner:
    // Address::ZERO }));
    assert!(err.downcast_ref::<ContractInitializationError>().is_some());

    Ok(())
}

#[e2e::test]
async fn transfers_ownership(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;
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

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(bob_addr))
        .deploy()
        .await?
        .contract_address;
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;
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
    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;
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

    let contract_addr = alice
        .as_deployer()
        .with_constructor(ctr(alice_addr))
        .deploy()
        .await?
        .contract_address;
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
