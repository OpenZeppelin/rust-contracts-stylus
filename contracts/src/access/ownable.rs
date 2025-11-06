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

use alloy_primitives::{aliases::B32, Address};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::MethodError, evm, msg, prelude::*, storage::StorageAddress,
};

use crate::utils::introspection::erc165::IErc165;

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
        /// The owner is not a valid owner account. (eg. [`Address::ZERO`])
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
    /// The owner is not a valid owner account. (eg. [`Address::ZERO`])
    InvalidOwner(OwnableInvalidOwner),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
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
    #[must_use]
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
    /// * [`OwnableInvalidOwner`] - If `new_owner` is the [`Address::ZERO`].
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
#[implements(IOwnable<Error = Error>, IErc165)]
impl Ownable {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `initial_owner` - The initial owner of this contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidOwner`] - If initial owner is [`Address::ZERO`].
    #[constructor]
    pub fn constructor(&mut self, initial_owner: Address) -> Result<(), Error> {
        if initial_owner.is_zero() {
            return Err(Error::InvalidOwner(OwnableInvalidOwner {
                owner: Address::ZERO,
            }));
        }
        self._transfer_ownership(initial_owner);
        Ok(())
    }
}

#[public]
impl IOwnable for Ownable {
    type Error = Error;

    fn owner(&self) -> Address {
        self.owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        self.transfer_ownership(new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        self.renounce_ownership()
    }
}

impl Ownable {
    /// Returns the address of the current owner.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    pub fn owner(&self) -> Address {
        self.owner.get()
    }

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
    /// * [`Error::InvalidOwner`] - If `new_owner` is the [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`OwnershipTransferred`].
    pub fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Error> {
        self.only_owner()?;

        if new_owner.is_zero() {
            return Err(Error::InvalidOwner(OwnableInvalidOwner {
                owner: Address::ZERO,
            }));
        }

        self._transfer_ownership(new_owner);

        Ok(())
    }

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
    pub fn renounce_ownership(&mut self) -> Result<(), Error> {
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

#[public]
impl IErc165 for Ownable {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IOwnable>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::*;
    use stylus_sdk::{alloy_primitives::Address, prelude::*};

    use super::*;
    use crate::utils::introspection::erc165::IErc165;

    unsafe impl TopLevelStorage for Ownable {}

    #[motsu::test]
    fn constructor(contract: Contract<Ownable>, alice: Address) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        let owner = contract.sender(alice).owner();
        assert_eq!(owner, alice);

        contract.assert_emitted(&OwnershipTransferred {
            previous_owner: Address::ZERO,
            new_owner: alice,
        });
    }

    #[motsu::test]
    fn constructor_reverts_when_invalid_owner(
        contract: Contract<Ownable>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            .constructor(Address::ZERO)
            .motsu_expect_err("should revert");
        assert!(
            matches!(err, Error::InvalidOwner(OwnableInvalidOwner { owner }) if owner.is_zero())
        );
    }

    #[motsu::test]
    fn transfers_ownership(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        contract
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("should transfer ownership");
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, bob);

        contract.assert_emitted(&OwnershipTransferred {
            previous_owner: alice,
            new_owner: bob,
        });
    }

    #[motsu::test]
    fn prevents_non_owners_from_transferring(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();

        let err =
            contract.sender(alice).transfer_ownership(bob).motsu_unwrap_err();

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(OwnableUnauthorizedAccount { account })
                if account == alice
        ));
    }

    #[motsu::test]
    fn prevents_reaching_stuck_state(
        contract: Contract<Ownable>,
        alice: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        let err = contract
            .sender(alice)
            .transfer_ownership(Address::ZERO)
            .motsu_unwrap_err();

        assert!(matches!(
            err,
            Error::InvalidOwner(OwnableInvalidOwner { owner }) if owner.is_zero()
        ));
    }

    #[motsu::test]
    fn loses_ownership_after_renouncing(
        contract: Contract<Ownable>,
        alice: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        contract
            .sender(alice)
            .renounce_ownership()
            .motsu_expect("should renounce ownership");
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, Address::ZERO);

        contract.assert_emitted(&OwnershipTransferred {
            previous_owner: alice,
            new_owner: Address::ZERO,
        });
    }

    #[motsu::test]
    fn prevents_non_owners_from_renouncing(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();

        let err =
            contract.sender(alice).renounce_ownership().motsu_unwrap_err();

        assert!(matches!(
            err,
            Error::UnauthorizedAccount(OwnableUnauthorizedAccount { account })
                if account == alice
        ));
    }

    #[motsu::test]
    fn recovers_access_using_internal_transfer(
        contract: Contract<Ownable>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();

        contract.sender(alice)._transfer_ownership(bob);
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Ownable as IOwnable>::interface_id();
        let expected: B32 = 0xe083076_u32.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Ownable>, alice: Address) {
        assert!(contract
            .sender(alice)
            .supports_interface(<Ownable as IOwnable>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Ownable as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
}
