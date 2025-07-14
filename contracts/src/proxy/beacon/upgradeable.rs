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
        /// Indicates an error related to the fact that the `implementation`
        /// of the beacon is invalid.
        ///
        /// * `implementation` - Address of the invalid implementation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error BeaconInvalidImplementation(address implementation);

        /// Emitted when the implementation returned by the beacon is changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Upgraded(address indexed implementation);
    }
}

/// An [`Erc1967Utils`] error.
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

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

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
/// [BeaconProxy][BeaconProxy] to determine their implementation contract, which
/// is where they will delegate all function calls.
///
/// An owner is able to change the implementation the beacon points to, thus
/// upgrading the proxies that use this beacon.
///
/// [BeaconProxy]: crate::proxy::beacon::BeaconProxy
pub trait IUpgradeableBeacon: IBeacon + IOwnable {
    // TODO: fn interface_id() -> FixedBytes<4>;

    /// Upgrades the beacon to a new implementation.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation.
    fn upgrade_to(
        &mut self,
        new_implementation: Address,
    ) -> Result<(), stylus_sdk::call::Error>;
}

/// State of an [`UpgradeableBeacon`] contract.
#[storage]
pub struct UpgradeableBeacon {
    /// The address of the implementation contract.
    implementation: StorageAddress,
    /// The [`Ownable`] contract that owns the beacon.
    ownable: Ownable,
}

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

impl IBeacon for UpgradeableBeacon {
    fn implementation(&self) -> Result<Address, stylus_sdk::call::Error> {
        Ok(self.implementation.get())
    }
}

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
