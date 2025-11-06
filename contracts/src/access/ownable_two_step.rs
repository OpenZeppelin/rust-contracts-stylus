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

use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{evm, msg, prelude::*, storage::StorageAddress};

use crate::{
    access::ownable::{self, Ownable},
    utils::introspection::erc165::IErc165,
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

/// State of an [`Ownable2Step`] contract.
#[storage]
pub struct Ownable2Step {
    /// [`Ownable`] contract.
    /// We leave the parent [`Ownable`] contract instance public, so that
    /// inheriting contract has access to its internal functions.
    pub ownable: Ownable,
    /// Pending owner of the contract.
    pub(crate) pending_owner: StorageAddress,
}

/// Interface for an [`Ownable2Step`] contract.
#[interface_id]
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
    /// Setting `new_owner` to [`Address::ZERO`] is allowed; this can be used
    /// to cancel an initiated ownership transfer.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_owner` - The next owner of this contract.
    ///
    /// # Errors
    ///
    /// * [`ownable::Error::UnauthorizedAccount`] - If called by any account
    ///   other than the owner.
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
    /// * [`ownable::Error::UnauthorizedAccount`] - If called by any account
    ///   other than the pending owner.
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
    /// * [`ownable::Error::UnauthorizedAccount`] - If not called by the owner.
    ///
    /// # Events
    ///
    /// * [`crate::access::ownable::OwnershipTransferred`].
    fn renounce_ownership(&mut self) -> Result<(), Self::Error>;
}

#[public]
#[implements(IOwnable2Step<Error = ownable::Error>, IErc165)]
impl Ownable2Step {
    /// See [`Ownable::constructor`].
    #[allow(clippy::missing_errors_doc)]
    #[constructor]
    pub fn constructor(
        &mut self,
        initial_owner: Address,
    ) -> Result<(), ownable::Error> {
        self.ownable.constructor(initial_owner)
    }
}

#[public]
impl IOwnable2Step for Ownable2Step {
    type Error = ownable::Error;

    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn pending_owner(&self) -> Address {
        self.pending_owner.get()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        self.ownable.only_owner()?;
        self.pending_owner.set(new_owner);

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
            return Err(ownable::Error::UnauthorizedAccount(
                ownable::OwnableUnauthorizedAccount { account: sender },
            ));
        }
        self._transfer_ownership(sender);
        Ok(())
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        self.ownable.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }
}

// This is implemented so that [`Ownable2Step`] could be passed to functions
// expecting [`ownable::IOwnable`].
impl ownable::IOwnable for Ownable2Step {
    type Error = ownable::Error;

    fn owner(&self) -> Address {
        IOwnable2Step::owner(self)
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        IOwnable2Step::transfer_ownership(self, new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        IOwnable2Step::renounce_ownership(self)
    }
}

impl Ownable2Step {
    /// Transfers ownership of the contract to a new account (`new_owner`) and
    /// sets [`Self::pending_owner`] to [`Address::ZERO`] to avoid situations
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
        self.pending_owner.set(Address::ZERO);
        self.ownable._transfer_ownership(new_owner);
    }
}

#[public]
impl IErc165 for Ownable2Step {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IOwnable2Step>::interface_id() == interface_id
            || self.ownable.supports_interface(interface_id)
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::{Contract, ResultExt};
    use stylus_sdk::{alloy_primitives::Address, prelude::*};

    use super::*;

    unsafe impl TopLevelStorage for Ownable2Step {}

    #[motsu::test]
    fn reads_owner(contract: Contract<Ownable2Step>, alice: Address) {
        contract.sender(alice).constructor(alice).motsu_unwrap();
        let owner = contract.sender(alice).owner();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn reads_pending_owner(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).pending_owner.set(bob);

        let pending_owner = contract.sender(alice).pending_owner();
        assert_eq!(pending_owner, bob);
    }

    #[motsu::test]
    fn initiates_ownership_transfer(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        contract
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("should initiate ownership transfer");

        assert_eq!(contract.sender(alice).owner(), alice);
    }

    #[motsu::test]
    fn prevents_non_owners_from_initiating_transfer(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();

        let err =
            contract.sender(alice).transfer_ownership(dave).motsu_unwrap_err();
        assert!(matches!(
            err,
            ownable::Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account
            }) if account == alice
        ));
    }

    #[motsu::test]
    fn accepts_ownership(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();
        contract.sender(alice).pending_owner.set(alice);

        contract
            .sender(alice)
            .accept_ownership()
            .motsu_expect("should accept ownership");
        assert_eq!(contract.sender(alice).owner(), alice);
        assert_eq!(contract.sender(alice).pending_owner(), Address::ZERO);
    }

    #[motsu::test]
    fn prevents_non_pending_owner_from_accepting(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();
        contract.sender(alice).pending_owner.set(dave);

        let err = contract.sender(alice).accept_ownership().motsu_unwrap_err();
        assert!(matches!(
            err,
            ownable::Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account
            }) if account == alice
        ));
    }

    #[motsu::test]
    fn completes_two_step_ownership_transfer(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        contract
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("should initiate ownership transfer");
        assert_eq!(contract.sender(alice).pending_owner(), bob);

        contract
            .sender(bob)
            .accept_ownership()
            .motsu_expect("should accept ownership");

        assert_eq!(contract.sender(alice).owner(), bob);
        assert_eq!(contract.sender(alice).pending_owner(), Address::ZERO);
    }

    #[motsu::test]
    fn renounces_ownership(contract: Contract<Ownable2Step>, alice: Address) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        contract
            .sender(alice)
            .renounce_ownership()
            .motsu_expect("should renounce ownership");
        assert_eq!(contract.sender(alice).owner(), Address::ZERO);
    }

    #[motsu::test]
    fn prevents_non_owners_from_renouncing(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(bob).motsu_unwrap();

        let err =
            contract.sender(alice).renounce_ownership().motsu_unwrap_err();
        assert!(matches!(
            err,
            ownable::Error::UnauthorizedAccount(ownable::OwnableUnauthorizedAccount {
                account
            }) if account == alice
        ));
    }

    #[motsu::test]
    fn cancels_transfer_on_renounce(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();
        contract.sender(alice).pending_owner.set(bob);

        contract
            .sender(alice)
            .renounce_ownership()
            .motsu_expect("should renounce ownership");
        assert_eq!(contract.sender(alice).owner(), Address::ZERO);
        assert_eq!(contract.sender(alice).pending_owner(), Address::ZERO);
    }

    #[motsu::test]
    fn allows_owner_to_cancel_transfer(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();
        contract.sender(alice).pending_owner.set(bob);

        contract
            .sender(alice)
            .transfer_ownership(Address::ZERO)
            .motsu_expect("should cancel transfer");
        assert_eq!(contract.sender(alice).pending_owner(), Address::ZERO);
        assert_eq!(contract.sender(alice).owner(), alice);
    }

    #[motsu::test]
    fn allows_owner_to_overwrite_transfer(
        contract: Contract<Ownable2Step>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract.sender(alice).constructor(alice).motsu_unwrap();

        contract
            .sender(alice)
            .transfer_ownership(bob)
            .motsu_expect("should initiate ownership transfer");
        assert_eq!(contract.sender(alice).pending_owner(), bob);

        contract
            .sender(alice)
            .transfer_ownership(dave)
            .motsu_expect("should overwrite transfer");
        assert_eq!(contract.sender(alice).pending_owner(), dave);
        assert_eq!(contract.sender(alice).owner(), alice);
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Ownable2Step as IOwnable2Step>::interface_id();
        let expected: B32 = 0x94be5999_u32.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Ownable2Step>, alice: Address) {
        assert!(contract.sender(alice).supports_interface(
            <Ownable2Step as IOwnable2Step>::interface_id()
        ));
        assert!(
            contract.sender(alice).supports_interface(
                <Ownable as ownable::IOwnable>::interface_id()
            )
        );
        assert!(contract
            .sender(alice)
            .supports_interface(<Ownable2Step as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
}
