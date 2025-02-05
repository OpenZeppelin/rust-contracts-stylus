//! Contract module which provides an access control mechanism, where
//! there is an account (an owner) that can be granted exclusive access to
//! specific functions.
//!
//! This extension of the `Ownable` contract includes a two-step mechanism to
//! transfer ownership, where the new owner must call
//! [`Ownable2Step::accept_ownership`] in order to replace the old one. This can
//! help prevent common mistakes, such as transfers of ownership to
//! incorrect accounts, or to contracts that are unable to interact with the
//! permission system.
//!
//! The initial owner is set to the address provided by the deployer. This can
//! later be changed with [`Ownable2Step::transfer_ownership`] and
//! [`Ownable2Step::accept_ownership`].
//!
//! This module uses [`Ownable`] as a member, and makes all its public functions
//! available.

use alloc::vec::Vec;

use alloy_primitives::Address;
pub use sol::*;
use stylus_sdk::{
    evm, msg,
    prelude::storage,
    storage::StorageAddress,
    stylus_proc::{public, SolidityError},
};

use crate::access::ownable::{
    Error as OwnableError, IOwnable, Ownable, OwnableUnauthorizedAccount,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when ownership transfer starts.
        ///
        /// * `previous_owner` - Address of the previous owner.
        /// * `new_owner` - Address of the new owner, to which the ownership
        ///   will be transferred.
        event OwnershipTransferStarted(
            address indexed previous_owner,
            address indexed new_owner
        );

    }
}

/// An error that occurred in the implementation of an [`Ownable2Step`]
/// contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Ownable`] contract.
    Ownable(OwnableError),
}

/// State of an [`Ownable2Step`] contract.
#[storage]
pub struct Ownable2Step {
    /// [`Ownable`] contract.
    #[allow(clippy::used_underscore_binding)]
    pub _ownable: Ownable,
    /// Pending owner of the contract.
    #[allow(clippy::used_underscore_binding)]
    pub _pending_owner: StorageAddress,
}

/// Interface for an [`Ownable2Step`] contract.
pub trait IOwnable2Step {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the current owner.
    ///
    /// Re-export of [`Ownable::owner`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn owner(&self) -> Address;

    /// Returns the address of the pending owner.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn pending_owner(&self) -> Address;

    /// Starts the ownership transfer of the contract to a new account.
    /// Replaces the pending transfer if there is one. Can only be called by the
    /// current owner.
    ///
    /// Setting `new_owner` to `Address::ZERO` is allowed; this can be used
    /// to cancel an initiated ownership transfer.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// * [`OwnableError::UnauthorizedAccount`] - If called by any account other
    ///   than the owner.
    ///
    /// # Events
    ///
    /// * [`OwnershipTransferStarted`].
    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error>;

    /// Accepts the ownership of the contract.
    /// Can only be called by the pending owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`OwnableError::UnauthorizedAccount`] - If called by any account other
    ///   than the pending owner.
    ///
    /// # Events
    ///
    /// * [`crate::access::ownable::OwnershipTransferred`].
    fn accept_ownership(&mut self) -> Result<(), Self::Error>;

    /// Leaves the contract without owner. It will not be possible to call
    /// [`Ownable::only_owner`] functions. Can only be called by the current
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
    /// * [`OwnableError::UnauthorizedAccount`] - If not called by the owner.
    ///
    /// # Events
    ///
    /// * [`crate::access::ownable::OwnershipTransferred`].
    fn renounce_ownership(&mut self) -> Result<(), Self::Error>;
}

#[public]
impl IOwnable2Step for Ownable2Step {
    type Error = Error;

    fn owner(&self) -> Address {
        self._ownable.owner()
    }

    fn pending_owner(&self) -> Address {
        self._pending_owner.get()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        self._ownable.only_owner()?;
        self._pending_owner.set(new_owner);

        let current_owner = self.owner();
        evm::log(OwnershipTransferStarted {
            previous_owner: current_owner,
            new_owner,
        });
        Ok(())
    }

    fn accept_ownership(&mut self) -> Result<(), Self::Error> {
        let sender = msg::sender();
        let pending_owner = self.pending_owner();
        if sender != pending_owner {
            return Err(OwnableError::UnauthorizedAccount(
                OwnableUnauthorizedAccount { account: sender },
            )
            .into());
        }
        self._transfer_ownership(sender);
        Ok(())
    }

    fn renounce_ownership(&mut self) -> Result<(), Error> {
        self._ownable.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }
}

impl Ownable2Step {
    /// Transfers ownership of the contract to a new account (`new_owner`) and
    /// sets [`Self::pending_owner`] to `Address::ZERO` to avoid situations
    /// where the transfer has been completed or the current owner renounces,
    /// but [`Self::pending_owner`] can still accept ownership.
    ///
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - Account that's gonna be the next owner.
    ///
    /// # Events
    ///
    /// * [`crate::access::ownable::OwnershipTransferred`].
    fn _transfer_ownership(&mut self, new_owner: Address) {
        self._pending_owner.set(Address::ZERO);
        self._ownable._transfer_ownership(new_owner);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address};
    use stylus_sdk::msg;

    use super::{Error, IOwnable2Step, Ownable2Step, OwnableError};

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const BOB: Address = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    #[motsu::test]
    fn reads_owner(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());
        let owner = contract.owner();
        assert_eq!(owner, msg::sender());
    }

    #[motsu::test]
    fn reads_pending_owner(contract: Ownable2Step) {
        contract._pending_owner.set(ALICE);
        let pending_owner = contract.pending_owner();
        assert_eq!(pending_owner, ALICE);
    }

    #[motsu::test]
    fn initiates_ownership_transfer(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());

        contract
            .transfer_ownership(ALICE)
            .expect("should initiate ownership transfer");
        let pending_owner = contract._pending_owner.get();
        assert_eq!(pending_owner, ALICE);
        assert_eq!(contract.owner(), msg::sender());
    }

    #[motsu::test]
    fn prevents_non_owners_from_initiating_transfer(contract: Ownable2Step) {
        contract._ownable._owner.set(ALICE);

        let err = contract.transfer_ownership(BOB).unwrap_err();
        assert!(matches!(
            err,
            Error::Ownable(OwnableError::UnauthorizedAccount(_))
        ));
    }

    #[motsu::test]
    fn accepts_ownership(contract: Ownable2Step) {
        contract._ownable._owner.set(ALICE);
        contract._pending_owner.set(msg::sender());

        contract.accept_ownership().expect("should accept ownership");
        assert_eq!(contract.owner(), msg::sender());
        assert_eq!(contract.pending_owner(), Address::ZERO);
    }

    #[motsu::test]
    fn prevents_non_pending_owner_from_accepting(contract: Ownable2Step) {
        contract._ownable._owner.set(ALICE);
        contract._pending_owner.set(BOB);

        let err = contract.accept_ownership().unwrap_err();
        assert!(matches!(
            err,
            Error::Ownable(OwnableError::UnauthorizedAccount(_))
        ));
    }

    #[motsu::test]
    fn completes_two_step_ownership_transfer(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());

        contract
            .transfer_ownership(ALICE)
            .expect("should initiate ownership transfer");
        assert_eq!(contract.pending_owner(), ALICE);

        // Simulate ALICE accepting ownership, since we cannot set `msg::sender`
        // in tests yet.
        contract._pending_owner.set(msg::sender());
        contract.accept_ownership().expect("should accept ownership");

        assert_eq!(contract.owner(), msg::sender());
        assert_eq!(contract.pending_owner(), Address::ZERO);
    }

    #[motsu::test]
    fn renounces_ownership(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());

        contract.renounce_ownership().expect("should renounce ownership");
        assert_eq!(contract.owner(), Address::ZERO);
    }

    #[motsu::test]
    fn prevents_non_owners_from_renouncing(contract: Ownable2Step) {
        contract._ownable._owner.set(ALICE);

        let err = contract.renounce_ownership().unwrap_err();
        assert!(matches!(
            err,
            Error::Ownable(OwnableError::UnauthorizedAccount(_))
        ));
    }

    #[motsu::test]
    fn cancels_transfer_on_renounce(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());
        contract._pending_owner.set(ALICE);

        contract.renounce_ownership().expect("should renounce ownership");
        assert_eq!(contract.owner(), Address::ZERO);
        assert_eq!(contract.pending_owner(), Address::ZERO);
    }

    #[motsu::test]
    fn allows_owner_to_cancel_transfer(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());
        contract._pending_owner.set(ALICE);

        contract
            .transfer_ownership(Address::ZERO)
            .expect("should cancel transfer");
        assert_eq!(contract.pending_owner(), Address::ZERO);
        assert_eq!(contract.owner(), msg::sender());
    }

    #[motsu::test]
    fn allows_owner_to_overwrite_transfer(contract: Ownable2Step) {
        contract._ownable._owner.set(msg::sender());

        contract
            .transfer_ownership(ALICE)
            .expect("should initiate ownership transfer");
        assert_eq!(contract.pending_owner(), ALICE);

        contract.transfer_ownership(BOB).expect("should overwrite transfer");
        assert_eq!(contract.pending_owner(), BOB);
        assert_eq!(contract.owner(), msg::sender());
    }
}
