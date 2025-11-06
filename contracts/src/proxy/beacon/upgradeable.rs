//! Implementation of the [`UpgradeableBeacon`] contract.

use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
pub use sol::*;
use stylus_sdk::{call::MethodError, evm, prelude::*, storage::StorageAddress};

use crate::{
    access::ownable::{self, IOwnable, Ownable},
    proxy::beacon::IBeacon,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the implementation returned by the beacon is changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Upgraded(address indexed implementation);
    }

    sol! {
        /// Indicates an error related to the fact that the `implementation`
        /// of the beacon is invalid.
        ///
        /// * `implementation` - Address of the invalid implementation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error BeaconInvalidImplementation(address implementation);
    }
}

/// An [`UpgradeableBeacon`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the fact that the `implementation`
    /// of the beacon is invalid.
    InvalidImplementation(BeaconInvalidImplementation),
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(ownable::OwnableUnauthorizedAccount),
    /// Indicates an error related to the fact that the `owner` of the
    /// beacon is invalid.
    InvalidOwner(ownable::OwnableInvalidOwner),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<ownable::Error> for Error {
    fn from(err: ownable::Error) -> Self {
        match err {
            ownable::Error::UnauthorizedAccount(err) => {
                Error::UnauthorizedAccount(err)
            }
            ownable::Error::InvalidOwner(err) => Error::InvalidOwner(err),
        }
    }
}

/// This contract is used in conjunction with one or more instances of
/// [`BeaconProxy`][BeaconProxy] to determine their implementation contract,
/// which is where they will delegate all function calls.
///
/// An owner is able to change the implementation the beacon points to, thus
/// upgrading the proxies that use this beacon.
///
/// [BeaconProxy]: super::BeaconProxy
pub trait IUpgradeableBeacon: IBeacon + IOwnable<Error = Vec<u8>> {
    /// Upgrades the beacon to a new implementation.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation.
    ///
    /// # Errors
    ///
    /// Implementing contracts should define their own error types for this
    /// function. Typically, errors may include:
    /// * The caller is not authorized to perform the upgrade.
    /// * The new implementation address is invalid (e.g., not a contract).
    /// * The upgrade operation failed for contract-specific reasons.
    ///
    /// The error should be encoded as a [`Vec<u8>`].
    fn upgrade_to(
        &mut self,
        new_implementation: Address,
    ) -> Result<(), Vec<u8>>;
}

/// State of an [`UpgradeableBeacon`] contract.
#[storage]
pub struct UpgradeableBeacon {
    /// The address of the implementation contract.
    implementation: StorageAddress,
    /// The [`Ownable`] contract that owns the beacon.
    ownable: Ownable,
}

unsafe impl TopLevelStorage for UpgradeableBeacon {}

#[public]
#[implements(IBeacon, IOwnable<Error = Error>)]
impl UpgradeableBeacon {
    /// Sets the address of the initial implementation, and the initial owner
    /// who can upgrade the beacon.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `implementation` - The address of the initial implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If `implementation` is not a
    ///   contract.
    /// * [`Error::UnauthorizedAccount`] - If the caller is not the owner.
    ///
    /// # Events
    ///
    /// * [`Upgraded`].
    #[constructor]
    pub fn constructor(
        &mut self,
        implementation: Address,
        initial_owner: Address,
    ) -> Result<(), Error> {
        self.ownable.constructor(initial_owner)?;
        self.set_implementation(implementation)?;
        Ok(())
    }

    /// Upgrades the beacon to a new implementation.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If `new_implementation` is not a
    ///   contract.
    /// * [`Error::UnauthorizedAccount`] - If the caller is not the owner.
    ///
    /// # Events
    ///
    /// * [`Upgraded`].
    pub fn upgrade_to(
        &mut self,
        new_implementation: Address,
    ) -> Result<(), Error> {
        self.ownable.only_owner()?;
        self.set_implementation(new_implementation)
    }

    /// Upgrades the beacon to a new implementation.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If `new_implementation` is not a
    ///   contract.
    ///
    /// # Events
    ///
    /// * [`Upgraded`].
    fn set_implementation(
        &mut self,
        new_implementation: Address,
    ) -> Result<(), Error> {
        if !new_implementation.has_code() {
            return Err(Error::InvalidImplementation(
                BeaconInvalidImplementation {
                    implementation: new_implementation,
                },
            ));
        }
        self.implementation.set(new_implementation);
        evm::log(Upgraded { implementation: new_implementation });
        Ok(())
    }
}

#[public]
impl IBeacon for UpgradeableBeacon {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        Ok(self.implementation.get())
    }
}

#[public]
impl IOwnable for UpgradeableBeacon {
    type Error = Error;

    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        Ok(self.ownable.renounce_ownership()?)
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::*;
    use stylus_sdk::alloy_primitives::Address;

    use super::*;
    use crate::proxy::{beacon::IBeacon, tests::Erc20Example};

    #[motsu::test]
    fn constructor(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());

        let owner = beacon.sender(alice).owner();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn constructor_with_invalid_implementation(
        beacon: Contract<UpgradeableBeacon>,
        alice: Address,
    ) {
        let invalid_address = alice;
        let err = beacon
            .sender(alice)
            .constructor(invalid_address, alice)
            .motsu_expect_err(
                "should fail when constructor has invalid implementation",
            );

        assert!(matches!(
            err,
            Error::InvalidImplementation(BeaconInvalidImplementation {
                implementation,
            }) if implementation == invalid_address
        ));
    }

    #[motsu::test]
    fn constructor_with_zero_owner(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        let err = beacon
            .sender(alice)
            .constructor(erc20.address(), Address::ZERO)
            .motsu_expect_err("should fail when constructor has zero owner");

        assert!(matches!(
            err,
            Error::InvalidOwner(ownable::OwnableInvalidOwner {
                owner,
            }) if owner.is_zero()
        ));
    }

    #[motsu::test]
    fn constructor_with_zero_implementation(
        beacon: Contract<UpgradeableBeacon>,
        alice: Address,
    ) {
        let err = beacon
            .sender(alice)
            .constructor(Address::ZERO, alice)
            .motsu_expect_err(
                "should fail when constructor has zero implementation",
            );

        assert!(matches!(
            err,
            Error::InvalidImplementation(BeaconInvalidImplementation {
                implementation,
            }) if implementation.is_zero()
        ));
    }

    #[motsu::test]
    fn upgrade_to_valid_implementation(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // upgrade to new implementation.
        beacon
            .sender(alice)
            .upgrade_to(erc20_2.address())
            .motsu_expect("should be able to upgrade to valid implementation");

        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20_2.address());

        // verify event was emitted.
        beacon.assert_emitted(&Upgraded { implementation: erc20_2.address() });
    }

    #[motsu::test]
    fn upgrade_to_invalid_implementation(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // try to upgrade to address with no code.
        let invalid_address = alice;
        let err =
            beacon.sender(alice).upgrade_to(invalid_address).motsu_expect_err(
                "should fail when upgrading to invalid implementation",
            );

        assert!(matches!(
            err,
            Error::InvalidImplementation(BeaconInvalidImplementation {
                implementation
            }) if implementation == invalid_address
        ));

        // implementation should remain unchanged.
        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn upgrade_to_unauthorized(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // try to upgrade from non-owner account.
        let err = beacon
            .sender(bob)
            .upgrade_to(erc20_2.address())
            .motsu_expect_err("should fail when called by non-owner");

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == bob
        ));

        // implementation should remain unchanged.
        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn upgrade_to_same_implementation(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // upgrade to the same implementation.
        beacon
            .sender(alice)
            .upgrade_to(erc20.address())
            .motsu_expect("should be able to upgrade to same implementation");

        // event should still be emitted.
        beacon.assert_emitted(&Upgraded { implementation: erc20.address() });

        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn upgrade_to_zero_address(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // try to upgrade to [`Address::ZERO`].
        let err = beacon
            .sender(alice)
            .upgrade_to(Address::ZERO)
            .motsu_expect_err("should fail when upgrading to zero address");

        assert!(matches!(
            err,
            Error::InvalidImplementation(BeaconInvalidImplementation {
                implementation,
            }) if implementation.is_zero()
        ));

        // implementation should remain unchanged.
        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20.address());
    }

    #[motsu::test]
    fn multiple_upgrades_emit_events(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        erc20_3: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // first upgrade.
        beacon
            .sender(alice)
            .upgrade_to(erc20_2.address())
            .motsu_expect("should be able to upgrade first time");

        beacon.assert_emitted(&Upgraded { implementation: erc20_2.address() });

        // second upgrade.
        beacon
            .sender(alice)
            .upgrade_to(erc20_3.address())
            .motsu_expect("should be able to upgrade second time");

        beacon.assert_emitted(&Upgraded { implementation: erc20_3.address() });

        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20_3.address());
    }

    #[motsu::test]
    fn transfer_ownership(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // transfer ownership to bob.
        beacon
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("should be able to transfer ownership");

        let owner = beacon.sender(alice).owner();
        assert_eq!(owner, bob);

        // bob should now be able to upgrade.
        beacon
            .sender(bob)
            .upgrade_to(erc20_2.address())
            .motsu_expect("new owner should be able to upgrade");

        // alice should not be able to upgrade.
        let err = beacon
            .sender(alice)
            .upgrade_to(erc20.address())
            .motsu_expect_err("should fail when called by non-owner");

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == alice
        ));
    }

    #[motsu::test]
    fn transfer_ownership_to_zero_address(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // try to transfer ownership to [`Address::ZERO`].
        let err = beacon
            .sender(alice)
            .transfer_ownership(Address::ZERO)
            .motsu_expect_err(
                "should fail when transferring ownership to zero address",
            );

        // the error should be from the ownable contract.
        assert!(matches!(
            err,
            Error::InvalidOwner(
                ownable::OwnableInvalidOwner {
                owner,
            }) if owner.is_zero(),
        ));

        // ownership should remain unchanged.
        let owner = beacon.sender(alice).owner();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfer_ownership_unauthorized(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // try to transfer ownership from non-owner account.
        let err = beacon
            .sender(bob)
            .transfer_ownership(charlie)
            .motsu_expect_err("should fail when called by non-owner");

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == bob
        ));

        // ownership should remain unchanged.
        let owner = beacon.sender(alice).owner();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn renounce_ownership(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // renounce ownership.
        beacon
            .sender(alice)
            .renounce_ownership()
            .motsu_expect("should be able to renounce ownership");

        let owner = beacon.sender(alice).owner();
        assert_eq!(owner, Address::ZERO);

        // no one should be able to upgrade now.
        let err = beacon
            .sender(alice)
            .upgrade_to(erc20_2.address())
            .motsu_expect_err("should fail when no owner exists");

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == alice
        ));
    }

    #[motsu::test]
    fn renounce_ownership_unauthorized(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // try to renounce ownership from non-owner account.
        let err = beacon
            .sender(bob)
            .renounce_ownership()
            .motsu_expect_err("should fail when called by non-owner");

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == bob
        ));

        // ownership should remain unchanged.
        let owner = beacon.sender(alice).owner();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn upgrade_after_ownership_transfer_chain(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        erc20_3: Contract<Erc20Example>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // alice transfers ownership to bob.
        beacon
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("should be able to transfer ownership to bob");

        // bob transfers ownership to charlie.
        beacon
            .sender(bob)
            .transfer_ownership(charlie)
            .motsu_expect("should be able to transfer ownership to charlie");

        // charlie should be able to upgrade.
        beacon
            .sender(charlie)
            .upgrade_to(erc20_2.address())
            .motsu_expect("charlie should be able to upgrade");

        let implementation = beacon
            .sender(alice)
            .implementation()
            .motsu_expect("should be able to get implementation");
        assert_eq!(implementation, erc20_2.address());

        // alice and bob should not be able to upgrade anymore.
        let err = beacon
            .sender(alice)
            .upgrade_to(erc20_3.address())
            .motsu_expect_err("alice should not be able to upgrade");
        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == alice
        ));

        let err = beacon
            .sender(bob)
            .upgrade_to(erc20_3.address())
            .motsu_expect_err("bob should not be able to upgrade");
        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == bob
        ));
    }

    #[motsu::test]
    fn upgrade_after_renounce_and_transfer(
        beacon: Contract<UpgradeableBeacon>,
        erc20: Contract<Erc20Example>,
        erc20_2: Contract<Erc20Example>,
        alice: Address,
    ) {
        beacon.sender(alice).constructor(erc20.address(), alice).motsu_unwrap();

        // alice renounces ownership.
        beacon
            .sender(alice)
            .renounce_ownership()
            .motsu_expect("should be able to renounce ownership");

        // no one should be able to upgrade.
        let err = beacon
            .sender(alice)
            .upgrade_to(erc20_2.address())
            .motsu_expect_err("should fail when no owner exists");
        assert!(matches!(
            err,
            Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account,
            }) if account == alice
        ));
    }
}
