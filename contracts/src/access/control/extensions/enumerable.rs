//! Extension of [`AccessControl`] that allows enumerating the members of each
//! role.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{prelude::*, storage::StorageMap};

use crate::{
    access::control::{self, AccessControl},
    utils::{
        introspection::erc165::IErc165, structs::enumerable_set::EnumerableSet,
    },
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Error returned when trying to query a role member at an invalid index.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error AccessControlEnumerableOutOfBounds(bytes32 role, uint256 index);
    }
}

/// An error that occurred in the implementation of an
/// [`AccessControlEnumerable`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Attempted to query a role member at an invalid index.
    OutOfBounds(AccessControlEnumerableOutOfBounds),
    /// The caller account is missing a role.
    UnauthorizedAccount(control::AccessControlUnauthorizedAccount),
    /// The caller of a afunction is not the expected one.
    BadConfirmation(control::AccessControlBadConfirmation),
}

impl From<control::Error> for Error {
    fn from(error: control::Error) -> Self {
        match error {
            control::Error::UnauthorizedAccount(error) => {
                Self::UnauthorizedAccount(error)
            }
            control::Error::BadConfirmation(error) => {
                Self::BadConfirmation(error)
            }
        }
    }
}
/// Interface for the AccessControlEnumerable extension
#[interface_id]
pub trait IAccessControlEnumerable {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// TODO: improve docs
    /// Returns one of the accounts that have `role`.
    /// `index` must be a value between 0 and {get_role_member_count},
    /// non-inclusive.
    fn get_role_member(
        &self,
        role: FixedBytes<32>,
        index: U256,
    ) -> Result<Address, Error>;

    /// TODO: improve docs
    /// Returns the number of accounts that have `role`.
    fn get_role_member_count(&self, role: FixedBytes<32>) -> U256;
}

/// State of an [`AccessControlEnumerable`] contract.
#[storage]
pub struct AccessControlEnumerable {
    /// TODO: docs
    role_members: StorageMap<FixedBytes<32>, EnumerableSet>,
}

unsafe impl TopLevelStorage for AccessControlEnumerable {}

#[public]
#[implements(IAccessControlEnumerable<Error = Error>, IErc165)]
impl AccessControlEnumerable {
    fn get_role_members(&self, role: FixedBytes<32>) -> Vec<Address> {
        self.role_members.get(role).values()
    }
}

#[public]
impl IAccessControlEnumerable for AccessControlEnumerable {
    type Error = Error;

    fn get_role_member(
        &self,
        role: FixedBytes<32>,
        index: U256,
    ) -> Result<Address, Self::Error> {
        let members = self.role_members.get(role);
        match members.at(index) {
            Some(member) => Ok(member),
            None => {
                Err(Error::OutOfBounds(AccessControlEnumerableOutOfBounds {
                    role,
                    index,
                }))
            }
        }
    }

    fn get_role_member_count(&self, role: FixedBytes<32>) -> U256 {
        self.role_members.get(role).length()
    }
}

impl AccessControlEnumerable {
    /// TODO: improve docs
    /// Grants `role` to `account`.
    ///
    /// If `account` had not been already granted `role`, emits a {RoleGranted}
    /// event.
    ///
    /// Requirements:
    /// - the caller must have ``role``'s admin role.
    pub fn _grant_role(
        &mut self,
        role: FixedBytes<32>,
        account: Address,
        access_control: &mut AccessControl,
    ) -> bool {
        let granted = access_control._grant_role(role, account);

        if granted {
            self.role_members.setter(role).add(account)
        }

        granted
    }

    /// TODO: improve docs
    /// Revokes `role` from `account`.
    ///
    /// If `account` had been granted `role`, emits a {RoleRevoked} event.
    ///
    /// Requirements:
    /// - the caller must have ``role``'s admin role.
    pub fn revoke_role(
        &mut self,
        role: FixedBytes<32>,
        account: Address,
        access_control: &mut AccessControl,
    ) -> bool {
        let revoked = access_control._revoke_role(role, account);

        if revoked {
            self.role_members.setter(role).remove(account);
        }

        revoked
    }
}

#[public]
impl IErc165 for AccessControlEnumerable {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IAccessControlEnumerable>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {

    use alloy_primitives::Address;
    use motsu::prelude::*;

    use super::*;

    const ROLE: [u8; 32] =
        keccak_const::Keccak256::new().update(b"ROLE").finalize();

    #[motsu::test]
    fn get_role_member_count_returns_zero_by_default(
        contract: Contract<AccessControlEnumerable>,
        alice: Address,
    ) {
        let count = contract.sender(alice).get_role_member_count(ROLE.into());
        assert_eq!(count, U256::ZERO);
    }

    #[motsu::test]
    fn get_role_member_reverts_when_empty_role(
        contract: Contract<AccessControlEnumerable>,
        alice: Address,
    ) {
        let role = ROLE.into();
        let index = U256::ZERO;

        let err = contract
            .sender(alice)
            .get_role_member(role, index)
            .motsu_expect_err("should return `Error::OutOfBounds`");

        assert!(matches!(
            err,
            Error::OutOfBounds(AccessControlEnumerableOutOfBounds { role: r, index: idx })
                if r == role && idx == index
        ));
    }

    /*
    #[motsu::test]
    fn can_enumerate_role_members(
        mut contract: Contract<AccessControlEnumerable>,
        admin: Address,
        alice: Address,
        bob: Address,
    ) {
        // Grant admin role first
        contract
            .sender(admin)
            .grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin)
            .unwrap();

        // Grant ROLE to alice and bob
        contract.sender(admin).grant_role(ROLE.into(), alice).unwrap();
        contract.sender(admin).grant_role(ROLE.into(), bob).unwrap();

        // Check member count
        let count = contract.get_role_member_count(ROLE.into());
        assert_eq!(count, U256::from(2));

        // Check members
        let member0 =
            contract.get_role_member(ROLE.into(), U256::ZERO).unwrap();
        let member1 =
            contract.get_role_member(ROLE.into(), U256::from(1)).unwrap();

        assert!(member0 == alice || member0 == bob);
        assert!(member1 == alice || member1 == bob);
        assert!(member0 != member1);
    }

    #[motsu::test]
    fn revoked_members_are_removed_from_enumeration(
        mut contract: Contract<AccessControlEnumerable>,
        admin: Address,
        alice: Address,
    ) {
        // Grant admin role first
        contract
            .sender(admin)
            .grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin)
            .unwrap();

        // Grant and revoke ROLE to alice
        contract.sender(admin).grant_role(ROLE.into(), alice).unwrap();
        contract.sender(admin).revoke_role(ROLE.into(), alice).unwrap();

        // Check member count is zero
        let count = contract.get_role_member_count(ROLE.into());
        assert_eq!(count, U256::ZERO);
    }

    #[motsu::test]
    fn supports_interface_returns_true_for_iaccess_control_enumerable(
        contract: Contract<AccessControlEnumerable>,
        alice: Address,
    ) {
        assert!(contract
            .sender(alice)
            .supports_interface(IAccessControlEnumerable::ID));
    }

    #[motsu::test]
    fn granting_role_twice_doesnt_duplicate_member(
        mut contract: Contract<AccessControlEnumerable>,
        admin: Address,
        alice: Address,
    ) {
        // Grant admin role first
        contract
            .sender(admin)
            .grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin)
            .unwrap();

        // Grant ROLE to alice twice
        contract.sender(admin).grant_role(ROLE.into(), alice).unwrap();
        contract.sender(admin).grant_role(ROLE.into(), alice).unwrap();

        // Check member count is still 1
        let count = contract.get_role_member_count(ROLE.into());
        assert_eq!(count, U256::from(1));

        // Check member is alice
        let member = contract.get_role_member(ROLE.into(), U256::ZERO).unwrap();
        assert_eq!(member, alice);
    }

    #[motsu::test]
    fn revoking_nonexistent_role_member_has_no_effect(
        mut contract: Contract<AccessControlEnumerable>,
        admin: Address,
        alice: Address,
    ) {
        // Grant admin role first
        contract
            .sender(admin)
            .grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin)
            .unwrap();

        // Try to revoke a role that alice doesn't have
        contract.sender(admin).revoke_role(ROLE.into(), alice).unwrap();

        // Check member count is still 0
        let count = contract.get_role_member_count(ROLE.into());
        assert_eq!(count, U256::ZERO);
    }
    */
}
