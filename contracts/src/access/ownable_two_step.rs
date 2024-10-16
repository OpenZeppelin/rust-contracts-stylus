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
//! This module is used through inheritance. It will make available all
//! functions from parent (`Ownable`).

use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_sdk::{
    evm, msg,
    stylus_proc::{public, sol_storage, SolidityError},
};

use crate::access::ownable::{
    Error as OwnableError, Ownable, OwnableUnauthorizedAccount,
};

// use super::ownable::{self, OwnableUnauthorizedAccount};

sol! {
    /// Emitted when ownership transfer starts.
    event OwnershipTransferStarted(address indexed previous_owner, address indexed new_owner);

    /// Emitted when ownership gets transferred between accounts.
    // TODO: Can we remove this and use the one in Ownable directly?
    event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);
}

// TODO: Since we are not introducing any new error type
// would we be better removing this and just relying on OwnableError?
/// An error that occurred in the implementation of an [`Ownable2Step`]
/// contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Ownable`] contract.
    Ownable(OwnableError),
}

sol_storage! {
    /// State of an `Ownable2Step` contract.
    pub struct Ownable2Step {
        /// [`Ownable`] contract.
        Ownable _ownable;
        /// Pending owner of the contract.
        address _pending_owner;
    }
}

#[public]
impl Ownable2Step {
    /// Returns the address of the current owner.
    pub fn owner(&self) -> Address {
        self._ownable.owner()
    }

    /// Returns the address of the pending owner.
    pub fn pending_owner(&self) -> Address {
        self._pending_owner.get()
    }

    /// Initiates the transfer of ownership to a new account (`new_owner`).
    /// Can only be called by the current owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// If called by any account other than the owner, then the error
    /// [`OwnableError::UnauthorizedAccount`] is returned.
    pub fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Error> {
        self._ownable.only_owner()?;
        self._pending_owner.set(new_owner);

        let current_owner = self.owner();
        evm::log(OwnershipTransferStarted {
            previous_owner: current_owner,
            new_owner,
        });
        Ok(())
    }

    /// Accepts the ownership of the contract. Can only be called by the
    /// pending owner.
    ///
    /// # Errors
    ///
    /// If called by any account other than the pending owner, then the error
    /// [`OwnableError::UnauthorizedAccount`] is returned.
    pub fn accept_ownership(&mut self) -> Result<(), Error> {
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

    /// Leaves the contract without owner. It will not be possible to call
    /// [`Self::only_owner`] functions. Can only be called by the current owner.
    ///
    /// NOTE: Renouncing ownership will leave the contract without an owner,
    /// thereby disabling any functionality that is only available to the owner.
    ///
    /// # Errors
    ///
    /// If not called by the owner, then the error
    /// [`OwnableError::UnauthorizedAccount`]
    pub fn renounce_ownership(&mut self) -> Result<(), OwnableError> {
        self._ownable.renounce_ownership()?;
        Ok(())
    }
}

impl Ownable2Step {
    /// Transfers ownership of the contract to a new account (`new_owner`) and
    /// sets [`Self::pending_owner`] to zero to avoid situations where the
    /// transfer has been completed or the current owner renounces, but
    /// [`Self::pending_owner`] can still accept ownership.
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - Account that's gonna be the next owner.
    fn _transfer_ownership(&mut self, new_owner: Address) {
        self._pending_owner.set(Address::ZERO);
        self._ownable._transfer_ownership(new_owner);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    // Test cases for Ownable2Step
}
