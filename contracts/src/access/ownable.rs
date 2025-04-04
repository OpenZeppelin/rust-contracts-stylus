//! Contract module which provides a basic access control mechanism, where
//! there is an account (an owner) that can be granted exclusive access to
//! specific functions.
//!
//! The initial owner is set to the address provided by the deployer. This can
//! later be changed with [`Ownable::transfer_ownership`].
//!
//! This module is used through inheritance. It will make available the
//! [`Ownable::only_owner`] function, which can be called to restrict operations
//! to the owner.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{evm, msg, prelude::*, storage::StorageAddress};

use crate::utils::introspection::erc165::{Erc165, IErc165};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when ownership gets transferred between accounts.
        ///
        /// * `previous_owner` - Address of the previous owner.
        /// * `new_owner` - Address of the new owner.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);
    }

    sol! {
        /// The caller account is not authorized to perform an operation.
        ///
        /// * `account` - Account that was found to not be authorized.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error OwnableUnauthorizedAccount(address account);
        /// The owner is not a valid owner account. (eg. `Address::ZERO`)
        ///
        /// * `owner` - Account that's not allowed to become the owner.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error OwnableInvalidOwner(address owner);
    }
}

/// An error that occurred in the implementation of an [`Ownable`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnableUnauthorizedAccount),
    /// The owner is not a valid owner account. (eg. `Address::ZERO`)
    InvalidOwner(OwnableInvalidOwner),
}

/// State of an [`Ownable`] contract.
#[storage]
pub struct Ownable {
    /// The current owner of this contract.
    pub(crate) owner: StorageAddress,
}

/// Interface for an [`Ownable`] contract.
#[interface_id]
pub trait IOwnable {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the current owner.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn owner(&self) -> Address;

    /// Transfers ownership of the contract to a new account (`new_owner`).
    /// Can only be called by the current owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// * [`OwnableInvalidOwner`] - If `new_owner` is the `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`OwnershipTransferred`].
    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error>;

    /// Leaves the contract without owner. It will not be possible to call
    /// functions that require `only_owner`. Can only be called by the current
    /// owner.
    ///
    /// NOTE: Renouncing ownership will leave the contract without an owner,
    /// thereby disabling any functionality that is only available to the owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedAccount`] - If not called by the owner.
    ///
    /// # Events
    ///
    /// * [`OwnershipTransferred`].
    fn renounce_ownership(&mut self) -> Result<(), Self::Error>;
}

#[public]
impl IOwnable for Ownable {
    type Error = Error;

    fn owner(&self) -> Address {
        self.owner.get()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        self.only_owner()?;

        if new_owner.is_zero() {
            return Err(Error::InvalidOwner(OwnableInvalidOwner {
                owner: Address::ZERO,
            }));
        }

        self._transfer_ownership(new_owner);

        Ok(())
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        self.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }
}

impl Ownable {
    /// Checks if the [`msg::sender`] is set as the owner.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedAccount`] - If called by any account other than
    ///   the owner.
    pub fn only_owner(&self) -> Result<(), Error> {
        let account = msg::sender();
        if self.owner() != account {
            return Err(Error::UnauthorizedAccount(
                OwnableUnauthorizedAccount { account },
            ));
        }

        Ok(())
    }

    /// Transfers ownership of the contract to a new account (`new_owner`).
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - Account that is going to be the next owner.
    ///
    /// # Events
    ///
    /// * [`OwnershipTransferred`].
    pub fn _transfer_ownership(&mut self, new_owner: Address) {
        let previous_owner = self.owner.get();
        self.owner.set(new_owner);
        evm::log(OwnershipTransferred { previous_owner, new_owner });
    }
}

impl IErc165 for Ownable {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IOwnable>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::TopLevelStorage;

    use super::{Error, IOwnable, Ownable};
    use crate::utils::introspection::erc165::IErc165;

    unsafe impl TopLevelStorage for Ownable {}

    #[motsu::test]
    fn reads_owner(contract: Contract<Ownable>, alice: Address) {
        contract.init(alice, |contract| contract.owner.set(alice));
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_ownership(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.init(alice, |contract| contract.owner.set(alice));

        contract
            .sender(alice)
            .transfer_ownership(bob)
            .expect("should transfer ownership");
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn prevents_non_onwers_from_transferring(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.init(alice, |contract| contract.owner.set(bob));

        let err = contract.sender(alice).transfer_ownership(bob).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn prevents_reaching_stuck_state(
        contract: Contract<Ownable>,
        alice: Address,
    ) {
        contract.init(alice, |contract| contract.owner.set(alice));

        let err = contract
            .sender(alice)
            .transfer_ownership(Address::ZERO)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidOwner(_)));
    }

    #[motsu::test]
    fn loses_ownership_after_renouncing(
        contract: Contract<Ownable>,
        alice: Address,
    ) {
        contract.init(alice, |contract| contract.owner.set(alice));

        contract
            .sender(alice)
            .renounce_ownership()
            .expect("should renounce ownership");
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, Address::ZERO);
    }

    #[motsu::test]
    fn prevents_non_owners_from_renouncing(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.init(alice, |contract| contract.owner.set(bob));

        let err = contract.sender(alice).renounce_ownership().unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn recovers_access_using_internal_transfer(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.init(alice, |contract| contract.owner.set(bob));

        contract.sender(alice)._transfer_ownership(bob);
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Ownable as IOwnable>::INTERFACE_ID;
        let expected = 0xe083076;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(Ownable::supports_interface(
            <Ownable as IOwnable>::INTERFACE_ID.into()
        ));
        assert!(Ownable::supports_interface(
            <Ownable as IErc165>::INTERFACE_ID.into()
        ));

        let fake_interface_id = 0x12345678u32;
        assert!(!Ownable::supports_interface(fake_interface_id.into()));
    }
}
