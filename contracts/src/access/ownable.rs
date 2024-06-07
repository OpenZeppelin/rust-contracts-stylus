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
use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_proc::SolidityError;
use stylus_sdk::{
    evm, msg,
    stylus_proc::{external, sol_storage},
};

sol! {
    /// Emitted when ownership gets transferred between accounts.
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

/// An error that occurred in the implementation of an [`Ownable`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnableUnauthorizedAccount),
    /// The owner is not a valid owner account. (eg. `Address::ZERO`)
    InvalidOwner(OwnableInvalidOwner),
}

sol_storage! {
    /// State of an `Ownable` contract.
    #[cfg_attr(test, derive(motsu::StylusDefault))]
    pub struct Ownable {
        /// The current owner of this contract.
        address _owner;
    }
}

#[external]
impl Ownable {
    /// Returns the address of the current owner.
    pub fn owner(&self) -> Address {
        self._owner.get()
    }

    /// Checks if the [`msg::sender`] is set as the owner.
    ///
    /// # Errors
    ///
    /// If called by any account other than the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn only_owner(&self) -> Result<(), Error> {
        let account = msg::sender();
        if self.owner() != account {
            return Err(Error::UnauthorizedAccount(
                OwnableUnauthorizedAccount { account },
            ));
        }

        Ok(())
    }

    /// Transfers ownership of the contract to a new account (`new_owner`). Can
    /// only be called by the current owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// If `new_owner` is the zero address, then the error
    /// [`Error::OwnableInvalidOwner`] is returned.
    pub fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Error> {
        self.only_owner()?;

        if new_owner == Address::ZERO {
            return Err(Error::InvalidOwner(OwnableInvalidOwner {
                owner: Address::ZERO,
            }));
        }

        self._transfer_ownership(new_owner);

        Ok(())
    }

    /// Leaves the contract without owner. It will not be possible to call
    /// [`only_owner`] functions. Can only be called by the current owner.
    ///
    /// NOTE: Renouncing ownership will leave the contract without an owner,
    /// thereby disabling any functionality that is only available to the owner.
    ///
    /// # Errors
    ///
    /// If not called by the owner, then the error
    /// [`Error::UnauthorizedAccount`] is returned.
    pub fn renounce_ownership(&mut self) -> Result<(), Error> {
        self.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }
}

impl Ownable {
    /// Transfers ownership of the contract to a new account (`new_owner`).
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - Account that's gonna be the next owner.
    pub fn _transfer_ownership(&mut self, new_owner: Address) {
        let previous_owner = self._owner.get();
        self._owner.set(new_owner);
        evm::log(OwnershipTransferred { previous_owner, new_owner });
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::{
        msg,
        storage::{StorageAddress, StorageType},
    };

    use super::{Error, Ownable};

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    #[motsu::test]
    fn rejects_zero_address_initial_owner(contract: Ownable) {
        // FIXME: Once constructors are supported this check should fail.
        assert_eq!(contract._owner.get(), Address::ZERO);
    }

    #[motsu::test]
    fn reads_owner(contract: Ownable) {
        contract._owner.set(msg::sender());
        let owner = contract.owner();
        assert_eq!(owner, msg::sender());
    }

    #[motsu::test]
    fn transfers_ownership(contract: Ownable) {
        contract._owner.set(msg::sender());

        contract.transfer_ownership(ALICE).expect("should transfer ownership");
        let owner = contract._owner.get();
        assert_eq!(owner, ALICE);
    }

    #[motsu::test]
    fn prevents_non_onwers_from_transferring(contract: Ownable) {
        // Alice must be set as owner, because we can't set the
        // `msg::sender` yet.
        contract._owner.set(ALICE);

        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let err = contract.transfer_ownership(bob).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn prevents_reaching_stuck_state(contract: Ownable) {
        contract._owner.set(msg::sender());

        let err = contract.transfer_ownership(Address::ZERO).unwrap_err();
        assert!(matches!(err, Error::InvalidOwner(_)));
    }

    #[motsu::test]
    fn loses_ownership_after_renouncing(contract: Ownable) {
        contract._owner.set(msg::sender());

        let _ = contract.renounce_ownership();
        let owner = contract._owner.get();
        assert_eq!(owner, Address::ZERO);
    }

    #[motsu::test]
    fn prevents_non_owners_from_renouncing(contract: Ownable) {
        // Alice must be set as owner, because we can't set the
        // `msg::sender` yet.
        contract._owner.set(ALICE);

        let err = contract.renounce_ownership().unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn recovers_access_using_internal_transfer(contract: Ownable) {
        contract._owner.set(ALICE);

        contract._transfer_ownership(ALICE);
        let owner = contract._owner.get();
        assert_eq!(owner, ALICE);
    }
}
