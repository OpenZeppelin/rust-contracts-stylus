#![cfg(feature = "e2e")]

use abi::Ownable::OwnershipTransferred;
use alloy::{
    primitives::Address,
    providers::Provider,
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
    sol_types::{SolConstructor, SolError, SolEvent},
};
use e2e::{receipt, send, Account, EventExt, Revert};
use eyre::Result;

use crate::abi::Ownable;

mod abi;

sol!("src/constructor.sol");

async fn deploy(account: &Account, owner: Address) -> eyre::Result<Address> {
    let args = OwnableExample::constructorCall { initialOwner: owner };
    let args = alloy::hex::encode(args.abi_encode());
    e2e::deploy(account.url(), &account.pk(), Some(args)).await
}

// ============================================================================
// Integration Tests: Ownable
// ============================================================================

#[e2e::test]
async fn constructs(alice: Account) -> Result<()> {
    let alice_addr = alice.address();
    let contract_addr = deploy(&alice, alice_addr).await?;
    let contract = Ownable::new(contract_addr, &alice.wallet);

    let Ownable::ownerReturn { owner } = contract.owner().call().await?;
    assert_eq!(owner, alice_addr);

    Ok(())
}

#[e2e::test]
async fn emits_ownership_transfer_during_construction(
    alice: Account,
) -> Result<()> {
    let alice_addr = alice.address();

    deploy(&alice, alice_addr).await?;

    let block = alice.wallet.get_block_number().await?;
    let filter = Filter::new()
        .event_signature(OwnershipTransferred::SIGNATURE_HASH)
        .from_block(BlockNumberOrTag::Number(block - 2));

    let logs = alice.wallet.get_logs(&filter).await?;
    let emitted = logs[0].log_decode::<OwnershipTransferred>()?.inner.data;
    let expected = OwnershipTransferred {
        previousOwner: Address::ZERO,
        newOwner: alice_addr,
    };
    assert_eq!(emitted, expected);

    Ok(())
}

#[e2e::test]
async fn rejects_zero_address_initial_owner(alice: Account) -> Result<()> {
    let err = deploy(&alice, Address::ZERO)
        .await
        .expect_err("should not deploy due to `OwnableInvalidOwner`");

    // TODO: Improve error check for contract deployments.
    // Issue: https://github.com/OpenZeppelin/rust-contracts-stylus/issues/128
    let err_string = format!("{:#?}", err);
    let expected = Ownable::OwnableInvalidOwner { owner: Address::ZERO };
    let expected = alloy::hex::encode(expected.abi_encode());

    assert!(err_string.contains(&expected));

    Ok(())
}

#[e2e::test]
async fn transfers_ownership(alice: Account, bob: Account) -> Result<()> {
    let alice_addr = alice.address();
    let bob_addr = bob.address();

    let contract_addr = deploy(&alice, alice_addr).await?;
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

    let contract_addr = deploy(&alice, bob_addr).await?;
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
    let contract_addr = deploy(&alice, alice_addr).await?;
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
    let contract_addr = deploy(&alice, alice_addr).await?;
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

    let contract_addr = deploy(&alice, alice_addr).await?;
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
