#![cfg(feature = "e2e")]

use abi::UpgradeableBeaconExample;
use alloy::primitives::Address;
use e2e::{constructor, receipt, send, Account, EventExt, Revert};
use eyre::Result;
use mock::erc20;

mod abi;
mod mock;

#[e2e::test]
async fn upgrade_to(alice: Account, bob: Account) -> Result<()> {
    let implementation_addr = erc20::deploy(&alice.wallet).await?;
    let receipt = alice
        .as_deployer()
        .with_constructor(constructor!(implementation_addr, alice.address()))
        .deploy()
        .await?;
    let contract =
        UpgradeableBeaconExample::new(receipt.contract_address, &alice.wallet);
    let contract_bob =
        UpgradeableBeaconExample::new(receipt.contract_address, &bob.wallet);

    // check initial state.
    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, implementation_addr);

    let owner = contract.owner().call().await?.owner;
    assert_eq!(owner, alice.address());

    assert!(receipt.emits(UpgradeableBeaconExample::Upgraded {
        implementation: implementation_addr,
    }));
    assert!(receipt.emits(UpgradeableBeaconExample::OwnershipTransferred {
        previousOwner: Address::ZERO,
        newOwner: alice.address(),
    }));

    // deploy new implementation.
    let new_implementation = erc20::deploy(&alice.wallet).await?;

    // check that bob cannot upgrade.
    let err = send!(contract_bob.upgradeTo(new_implementation))
        .expect_err("should revert on non-owner");
    assert!(err.reverted_with(
        UpgradeableBeaconExample::OwnableUnauthorizedAccount {
            sender: bob.address(),
        }
    ));

    // check that alice can upgrade.
    let receipt = receipt!(contract.upgradeTo(new_implementation))?;

    assert!(receipt.emits(UpgradeableBeaconExample::Upgraded {
        implementation: new_implementation,
    }));

    let implementation = contract.implementation().call().await?.implementation;
    assert_eq!(implementation, new_implementation);

    Ok(())
}
