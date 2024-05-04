//! Contract module which provides a basic access control mechanism, where
//! there is an account (an owner) that can be granted exclusive access to
//! specific functions.
//!
//! The initial owner is set to the address provided by the deployer. This can
//! later be changed with {transferOwnership}.
//!
//! This module is used through inheritance. It will make available the modifier
//! `onlyOwner`, which can be applied to your functions to restrict their use to
//! the owner.
use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_proc::SolidityError;
use stylus_sdk::{
    evm, msg,
    stylus_proc::{external, sol_storage},
};

sol! {
    /// Emitted when ownership gets transferred between accounts.
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
}

sol! {
     /// The caller account is not authorized to perform an operation.
    #[derive(Debug)]
    error OwnableUnauthorizedAccount(address account);
     /// The owner is not a valid owner account. (eg. `address(0)`)
    #[derive(Debug)]
    error OwnableInvalidOwner(address owner);

}

/// An error that occurred in the implementation of an `Ownable` contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The caller account is not authorized to perform an operation.
    UnauthorizedAccount(OwnableUnauthorizedAccount),
    /// The owner is not a valid owner account. (eg. `address(0)`)
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
    pub fn constructor(&mut self, initial_owner: Address) -> Result<(), Error> {
        if self._initialized.get() {
            panic!("Ownable has already been initialized");
        }

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

    /// Errors if called by any account other than the owner.
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
    /// Can only be called by the current owner.
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
    pub fn renounce_ownership(&mut self) -> Result<(), Error> {
        self.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }
}

impl Ownable {
    /// Transfers ownership of the contract to a new account (`new_owner`).
    /// Internal function without access restriction.
    pub fn _transfer_ownership(&mut self, new_owner: Address) {
        let old_owner = self._owner.get();
        self._owner.set(new_owner);

        evm::log(OwnershipTransferred {
            previousOwner: old_owner,
            newOwner: new_owner,
        });
    }
}
