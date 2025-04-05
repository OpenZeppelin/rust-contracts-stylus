//! Extension of AccessControl that allows enumerating the members of each role.
//!
//! This implements an optional extension of [`super::control::AccessControl`] that adds
//! enumerability of all accounts that have been granted a role.
//!
//! CAUTION: When using getRoleMember and getRoleMemberCount, make sure you perform
//! all queries on the same block to avoid inconsistencies.

use alloc::{vec, vec::Vec};
use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    prelude::*,
    storage::{StorageMap, StorageU256, StorageVec},
};

use super::control::{self, AccessControl, Error as AccessControlError, IAccessControl};
use crate::utils::introspection::erc165::{Erc165, IErc165};

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

/// An error that occurred in the implementation of an [`AccessControlEnumerable`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// An error occurred in the base AccessControl contract.
    AccessControl(AccessControlError),
    /// Attempted to query a role member at an invalid index.
    OutOfBounds(AccessControlEnumerableOutOfBounds),
}

/// Interface for the AccessControlEnumerable extension
#[interface_id]
pub trait IAccessControlEnumerable: IAccessControl {
    /// Returns one of the accounts that have `role`.
    /// `index` must be a value between 0 and {get_role_member_count}, non-inclusive.
    fn get_role_member(role: FixedBytes<32>, index: U256) -> Result<Address, Error>;

    /// Returns the number of accounts that have `role`.
    fn get_role_member_count(role: FixedBytes<32>) -> U256;
}

/// State for role enumeration functionality
#[storage]
#[derive(Debug)]
pub struct RoleEnumeration {
    /// Mapping from role to list of member addresses.
    /// This tracks all accounts that have been granted each role for enumeration purposes.
    role_members: StorageMap<FixedBytes<32>, StorageVec<Address>>,
}

/// State of an enumerable access control contract.
/// Combines the base AccessControl functionality with role enumeration capabilities.
#[storage]
#[derive(Debug)]
pub struct AccessControlEnumerable {
    /// Base access control functionality
    #[borrow]
    access: AccessControl,
    /// Role enumeration state
    #[borrow] 
    enumeration: RoleEnumeration,
}

#[external]
impl IAccessControlEnumerable for AccessControlEnumerable {
    fn get_role_member(&self, role: FixedBytes<32>, index: U256) -> Result<Address, Error> {
        let members = self.enumeration.role_members.get(role);
        if index >= U256::from(members.len()) {
            return Err(Error::OutOfBounds(AccessControlEnumerableOutOfBounds {
                role,
                index,
            }));
        }
        Ok(members[index.as_usize()])
    }

    fn get_role_member_count(&self, role: FixedBytes<32>) -> U256 {
        let members = self.enumeration.role_members.get(role);
        U256::from(members.len())
    }
}

impl AccessControlEnumerable {
    /// Grants `role` to `account`.
    ///
    /// If `account` had not been already granted `role`, emits a {RoleGranted} event.
    /// 
    /// Requirements:
    /// - the caller must have ``role``'s admin role.
    pub fn grant_role(&mut self, role: FixedBytes<32>, account: Address) -> Result<(), Error> {
        if self.access._grant_role(role, account)
            .map_err(Error::AccessControl)? 
        {
            let members = self.enumeration.role_members.get(role);
            members.push(account);
        }
        Ok(())
    }

    /// Revokes `role` from `account`.
    ///
    /// If `account` had been granted `role`, emits a {RoleRevoked} event.
    ///
    /// Requirements:
    /// - the caller must have ``role``'s admin role.
    pub fn revoke_role(&mut self, role: FixedBytes<32>, account: Address) -> Result<(), Error> {
        if self.access._revoke_role(role, account)
            .map_err(Error::AccessControl)?
        {
            let members = self.enumeration.role_members.get(role);
            // Find and remove account from members array
            for i in 0..members.len() {
                if members[i] == account {
                    // Move last member to the removed position
                    let last_idx = members.len() - 1;
                    if i != last_idx {
                        members[i] = members[last_idx];
                    }
                    members.pop();
                    break;
                }
            }
        }
        Ok(())
    }

    /// Returns true if this contract implements the interface defined by `interface_id`.
    pub fn supports_interface(&self, interface_id: [u8; 4]) -> bool {
        interface_id == IAccessControlEnumerable::ID || self.access.supports_interface(interface_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;
    use stylus_sdk::contract::Contract;

    const ROLE: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"ROLE")
        .finalize();

    #[motsu::test]
    fn get_role_member_count_returns_zero_by_default(
        contract: Contract<AccessControlEnumerable>,
        alice: Address,
    ) {
        let count = contract.sender(alice).get_role_member_count(ROLE.into());
        assert_eq!(count, U256::ZERO);
    }

    #[motsu::test]
    fn get_role_member_fails_for_empty_role(
        contract: Contract<AccessControlEnumerable>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            .get_role_member(ROLE.into(), U256::ZERO)
            .unwrap_err();
        assert!(matches!(err, Error::OutOfBounds(_)));
    }

    #[motsu::test]
    fn can_enumerate_role_members(
        mut contract: Contract<AccessControlEnumerable>,
        admin: Address,
        alice: Address,
        bob: Address,
    ) {
        // Grant admin role first
        contract.sender(admin).grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin).unwrap();
        
        // Grant ROLE to alice and bob
        contract.sender(admin).grant_role(ROLE.into(), alice).unwrap();
        contract.sender(admin).grant_role(ROLE.into(), bob).unwrap();

        // Check member count
        let count = contract.get_role_member_count(ROLE.into());
        assert_eq!(count, U256::from(2));

        // Check members
        let member0 = contract.get_role_member(ROLE.into(), U256::ZERO).unwrap();
        let member1 = contract.get_role_member(ROLE.into(), U256::from(1)).unwrap();
        
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
        contract.sender(admin).grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin).unwrap();
        
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
        assert!(contract.sender(alice).supports_interface(IAccessControlEnumerable::ID));
    }

    #[motsu::test]
    fn granting_role_twice_doesnt_duplicate_member(
        mut contract: Contract<AccessControlEnumerable>,
        admin: Address,
        alice: Address,
    ) {
        // Grant admin role first
        contract.sender(admin).grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin).unwrap();
        
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
        contract.sender(admin).grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin).unwrap();
        
        // Try to revoke a role that alice doesn't have
        contract.sender(admin).revoke_role(ROLE.into(), alice).unwrap();

        // Check member count is still 0
        let count = contract.get_role_member_count(ROLE.into());
        assert_eq!(count, U256::ZERO);
    }
} 