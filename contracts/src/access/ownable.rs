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
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
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
    pub struct Ownable {
        /// The current owner of this contract.
        address _owner;
        /// Initialization marker. If true this means that the constructor was
        /// called.
        ///
        /// This field should be unnecessary once constructors are supported in
        /// the SDK.
        bool _initialized;
    }
}

#[external]
impl Ownable {
    /// Initializes an [`Ownable`] instance with the given `initial_owner`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `initial_owner` - The initial owner of this contract.
    ///
    /// # Errors
    ///
    /// * If `initial_owner` is the zero address, then [`Error::InvalidOwner`]
    /// is returned.
    ///
    /// # Panics
    ///
    /// * If the contract is already initialized, then this function panics.
    /// This ensures the contract is constructed only once.
    pub fn constructor(&mut self, initial_owner: Address) -> Result<(), Error> {
        let is_initialized = self._initialized.get();
        assert!(!is_initialized, "Ownable has already been initialized");

        if initial_owner == Address::ZERO {
            return Err(Error::InvalidOwner(OwnableInvalidOwner {
                owner: Address::ZERO,
            }));
        }

        self._transfer_ownership(initial_owner);
        self._initialized.set(true);

        Ok(())
    }

    /// Returns the address of the current owner.
    pub fn owner(&self) -> Address {
        self._owner.get()
    }

    /// Checks if the [`msg::sender`] is set as the owner.
    ///
    /// # Errors
    ///
    /// * If called by any account other than the owner returns
    /// [`Error::UnauthorizedAccount`].
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
    /// * If `new_owner` is the zero address, then this function returns an
    /// [`Error::OwnableInvalidOwner`] error.
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
    /// * If not called by the owner, then an [`Error::UnauthorizedAccount`] is
    /// returned.
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
        let old_owner = self._owner.get();
        self._owner.set(new_owner);

        evm::log(OwnershipTransferred {
            previousOwner: old_owner,
            newOwner: new_owner,
        });
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::{
        msg,
        storage::{StorageAddress, StorageBool, StorageType},
    };

    use super::{Error, Ownable};

    impl Default for Ownable {
        fn default() -> Self {
            let root = U256::ZERO;
            Ownable {
                _owner: unsafe { StorageAddress::new(root, 0) },
                _initialized: unsafe {
                    StorageBool::new(root + U256::from(32), 0)
                },
            }
        }
    }

    #[grip::test]
    fn rejects_zero_address_initial_owner(contract: Ownable) {
        // FIXME: Once constructors are supported this check should fail.
        assert_eq!(contract._owner.get(), Address::ZERO);

        let err = contract.constructor(Address::ZERO).unwrap_err();
        assert!(matches!(err, Error::InvalidOwner(_)));
    }

    #[grip::test]
    #[should_panic = "Ownable has already been initialized"]
    fn initializes_owner_once(contract: Ownable) {
        let result = contract.constructor(msg::sender());
        assert!(result.is_ok());

        let owner = contract._owner.get();
        assert_eq!(owner, msg::sender());

        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let _ = contract.constructor(alice);
    }

    #[grip::test]
    fn reads_owner(contract: Ownable) {
        let result = contract.constructor(msg::sender());
        assert!(result.is_ok());

        let owner = contract.owner();
        assert_eq!(owner, msg::sender());
    }

    #[grip::test]
    fn transfers_ownership(contract: Ownable) {
        let result = contract.constructor(msg::sender());
        assert!(result.is_ok());

        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let _ = contract
            .transfer_ownership(alice)
            .expect("should transfer ownership");
        let owner = contract._owner.get();
        assert_eq!(owner, alice);
    }

    #[grip::test]
    fn prevents_non_onwers_from_transferring(contract: Ownable) {
        // Alice must be set as owner, because we can't set the msg::sender yet.
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let result = contract.constructor(alice);
        assert!(result.is_ok());

        let bob = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");
        let err = contract.transfer_ownership(bob).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[grip::test]
    fn prevents_reaching_stuck_state(contract: Ownable) {
        let result = contract.constructor(msg::sender());
        assert!(result.is_ok());

        let err = contract.transfer_ownership(Address::ZERO).unwrap_err();
        assert!(matches!(err, Error::InvalidOwner(_)));
    }

    #[grip::test]
    fn loses_ownership_after_renouncing(contract: Ownable) {
        let result = contract.constructor(msg::sender());
        assert!(result.is_ok());

        let _ = contract.renounce_ownership();
        let owner = contract._owner.get();
        assert_eq!(owner, Address::ZERO);
    }

    #[grip::test]
    fn prevents_non_owners_from_renouncing(contract: Ownable) {
        // Alice must be set as owner, because we can't set the msg::sender yet.
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let result = contract.constructor(alice);
        assert!(result.is_ok());

        let err = contract.renounce_ownership().unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[grip::test]
    fn recovers_access_using_internal_transfer(contract: Ownable) {
        let result = contract.constructor(msg::sender());
        assert!(result.is_ok());

        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        contract._transfer_ownership(alice);
        let owner = contract._owner.get();
        assert_eq!(owner, alice);
    }
}
