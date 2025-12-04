//! Extension of [`AccessControl`] that allows enumerating the members of each
//! role.

use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, B256, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{prelude::*, storage::StorageMap};

use crate::{
    access::control::AccessControl,
    utils::{
        introspection::erc165::IErc165, structs::enumerable_set::EnumerableSet,
    },
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The caller attempted to query a `role` member at an invalid `index`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error AccessControlEnumerableOutOfBounds(bytes32 role, uint256 index);
    }
}

/// An error that occurred in the implementation of an
/// [`AccessControlEnumerable`] contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The caller attempted to query a `role` member at an invalid `index`.
    OutOfBounds(AccessControlEnumerableOutOfBounds),
}

/// State of an [`AccessControlEnumerable`] contract.
#[storage]
pub struct AccessControlEnumerable {
    /// Role identifier -> [`EnumerableSet`] of accounts.
    pub(crate) role_members: StorageMap<B256, EnumerableSet<Address>>,
}

/// Interface for the [`AccessControlEnumerable`] extension.
#[interface_id]
pub trait IAccessControlEnumerable {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns one of the accounts that have `role`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    /// * `index` - The index of the account at `role`'s members list.
    ///
    /// # Errors
    ///
    /// * [`Error::OutOfBounds`] - If `index` is out of bounds.
    fn get_role_member(
        &self,
        role: B256,
        index: U256,
    ) -> Result<Address, Error>;

    /// Returns the number of accounts that have role.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    fn get_role_member_count(&self, role: B256) -> U256;
}

#[public]
#[implements(IAccessControlEnumerable<Error = Error>, IErc165)]
impl AccessControlEnumerable {
    /// Returns the members of `role`.
    ///
    /// # WARNING
    ///
    /// This operation will copy the entire storage to memory, which can be
    /// quite expensive. This is designed to mostly be used by view
    /// accessors that are queried without any gas fees. Developers should keep
    /// in mind that this function has an unbounded cost, and using it as
    /// part of a state-changing function may render the function uncallable
    /// if the set grows to a point where copying to memory consumes too much
    /// gas to fit in a block.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    #[must_use]
    pub fn get_role_members(&self, role: B256) -> Vec<Address> {
        self.role_members.get(role).values()
    }
}

#[public]
impl IAccessControlEnumerable for AccessControlEnumerable {
    type Error = Error;

    fn get_role_member(
        &self,
        role: B256,
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

    fn get_role_member_count(&self, role: B256) -> U256 {
        self.role_members.get(role).length()
    }
}

impl AccessControlEnumerable {
    /// Extension of [`AccessControl::_grant_role`] that adds the `account` to
    /// `role`'s members list.
    #[allow(clippy::missing_errors_doc)]
    pub fn _grant_role(
        &mut self,
        role: B256,
        account: Address,
        access_control: &mut AccessControl,
    ) -> bool {
        let granted = access_control._grant_role(role, account);

        if granted {
            self.role_members.setter(role).add(account);
        }

        granted
    }

    /// Extension of [`AccessControl::_revoke_role`] that removes the `account`
    /// from `role`'s members list.
    #[allow(clippy::missing_errors_doc)]
    pub fn _revoke_role(
        &mut self,
        role: B256,
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
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IAccessControlEnumerable>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {

    use alloy_primitives::{uint, Address};
    use motsu::prelude::*;
    use stylus_sdk::msg;

    use super::*;
    use crate::access::control::{self, IAccessControl};

    const ROLE: [u8; 32] =
        keccak_const::Keccak256::new().update(b"ROLE").finalize();

    #[storage]
    struct AccessControlEnumerableExample {
        access_control: AccessControl,
        enumerable: AccessControlEnumerable,
    }

    unsafe impl TopLevelStorage for AccessControlEnumerableExample {}

    #[public]
    #[implements(IAccessControl<Error = control::Error>, IAccessControlEnumerable<Error = Error>,  IErc165)]
    impl AccessControlEnumerableExample {
        fn get_role_members(&self, role: B256) -> Vec<Address> {
            self.enumerable.get_role_members(role)
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[public]
    impl IAccessControl for AccessControlEnumerableExample {
        type Error = control::Error;

        fn has_role(&self, role: B256, account: Address) -> bool {
            self.access_control.has_role(role, account)
        }

        fn only_role(&self, role: B256) -> Result<(), Self::Error> {
            self.access_control.only_role(role)
        }

        fn get_role_admin(&self, role: B256) -> B256 {
            self.access_control.get_role_admin(role)
        }

        fn grant_role(
            &mut self,
            role: B256,
            account: Address,
        ) -> Result<(), Self::Error> {
            let admin_role = self.get_role_admin(role);
            self.only_role(admin_role)?;
            self.enumerable._grant_role(
                role,
                account,
                &mut self.access_control,
            );
            Ok(())
        }

        fn revoke_role(
            &mut self,
            role: B256,
            account: Address,
        ) -> Result<(), Self::Error> {
            let admin_role = self.get_role_admin(role);
            self.only_role(admin_role)?;
            self.enumerable._revoke_role(
                role,
                account,
                &mut self.access_control,
            );
            Ok(())
        }

        fn renounce_role(
            &mut self,
            role: B256,
            confirmation: Address,
        ) -> Result<(), Self::Error> {
            if msg::sender() != confirmation {
                return Err(control::Error::BadConfirmation(
                    control::AccessControlBadConfirmation {},
                ));
            }

            self.enumerable._revoke_role(
                role,
                confirmation,
                &mut self.access_control,
            );
            Ok(())
        }
    }

    #[public]
    impl IAccessControlEnumerable for AccessControlEnumerableExample {
        type Error = Error;

        fn get_role_member(
            &self,
            role: B256,
            index: U256,
        ) -> Result<Address, Self::Error> {
            self.enumerable.get_role_member(role, index)
        }

        fn get_role_member_count(&self, role: B256) -> U256 {
            self.enumerable.get_role_member_count(role)
        }
    }

    #[public]
    impl IErc165 for AccessControlEnumerableExample {
        fn supports_interface(&self, interface_id: B32) -> bool {
            self.enumerable.supports_interface(interface_id)
                || self.access_control.supports_interface(interface_id)
        }
    }

    #[motsu::test]
    fn get_role_member_count_returns_zero_by_default(
        contract: Contract<AccessControlEnumerableExample>,
        alice: Address,
    ) {
        let count = contract.sender(alice).get_role_member_count(ROLE.into());
        assert_eq!(count, U256::ZERO);

        assert!(contract
            .sender(alice)
            .get_role_members(ROLE.into())
            .is_empty());
    }

    #[motsu::test]
    fn get_role_member_reverts_when_empty_role(
        contract: Contract<AccessControlEnumerableExample>,
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

        assert!(contract.sender(alice).get_role_members(role).is_empty());
    }

    #[motsu::test]
    fn can_enumerate_role_members(
        contract: Contract<AccessControlEnumerableExample>,
        admin: Address,
        alice: Address,
        bob: Address,
    ) {
        // Grant [`AccessControl::DEFAULT_ADMIN_ROLE`] to `admin`.
        contract
            .sender(admin)
            .access_control
            ._grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin);

        // Grant `ROLE` to `alice` and `bob`.
        contract
            .sender(admin)
            .grant_role(ROLE.into(), alice)
            .motsu_expect("should grant alice");

        contract
            .sender(admin)
            .grant_role(ROLE.into(), bob)
            .motsu_expect("should grant bob");

        // Check members count.
        assert_eq!(
            contract.sender(alice).get_role_member_count(ROLE.into()),
            uint!(2_U256)
        );

        // Check members.
        assert_eq!(
            contract
                .sender(alice)
                .get_role_member(ROLE.into(), U256::ZERO)
                .motsu_expect("should return alice"),
            alice
        );

        assert_eq!(
            contract
                .sender(alice)
                .get_role_member(ROLE.into(), U256::ONE)
                .motsu_expect("should return bob"),
            bob
        );

        assert_eq!(
            contract.sender(alice).get_role_members(ROLE.into()),
            vec![alice, bob]
        );
    }

    #[motsu::test]
    fn revoked_members_are_removed_from_enumeration(
        contract: Contract<AccessControlEnumerableExample>,
        admin: Address,
        alice: Address,
    ) {
        // Grant [`AccessControl::DEFAULT_ADMIN_ROLE`] to `admin`.
        contract
            .sender(admin)
            .access_control
            ._grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin);

        // Grant and revoke `ROLE` to `alice`.
        contract
            .sender(admin)
            .grant_role(ROLE.into(), alice)
            .motsu_expect("should grant alice");
        contract
            .sender(admin)
            .revoke_role(ROLE.into(), alice)
            .motsu_expect("should revoke alice");

        // Check member count.
        assert_eq!(
            contract.sender(alice).get_role_member_count(ROLE.into()),
            U256::ZERO
        );

        assert!(contract
            .sender(alice)
            .get_role_members(ROLE.into())
            .is_empty());
    }

    #[motsu::test]
    fn supports_interface_returns_true_for_iaccess_control_enumerable(
        contract: Contract<AccessControlEnumerableExample>,
        alice: Address,
    ) {
        assert!(contract
            .sender(alice)
            .supports_interface(<AccessControlEnumerableExample as IAccessControlEnumerable>::interface_id()));

        assert!(contract.sender(alice).supports_interface(
            <AccessControlEnumerableExample as IAccessControl>::interface_id()
        ));

        assert!(contract.sender(alice).supports_interface(
            <AccessControlEnumerableExample as IErc165>::interface_id()
        ));
    }

    #[motsu::test]
    fn granting_role_twice_does_not_duplicate_member(
        contract: Contract<AccessControlEnumerableExample>,
        admin: Address,
        alice: Address,
    ) {
        // Grant [`AccessControl::DEFAULT_ADMIN_ROLE`] to `admin`.
        contract
            .sender(admin)
            .access_control
            ._grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin);

        // Grant `ROLE` to `alice`.
        contract
            .sender(admin)
            .grant_role(ROLE.into(), alice)
            .motsu_expect("should grant alice");

        // Grant `ROLE` to `alice` again.
        contract
            .sender(admin)
            .grant_role(ROLE.into(), alice)
            .motsu_expect("should grant alice");

        // Check member count.
        assert_eq!(
            contract.sender(alice).get_role_member_count(ROLE.into()),
            U256::ONE
        );

        // Check member.
        let member = contract
            .sender(alice)
            .get_role_member(ROLE.into(), U256::ZERO)
            .motsu_expect("should return alice");
        assert_eq!(member, alice);

        assert_eq!(
            contract.sender(alice).get_role_members(ROLE.into()),
            vec![alice]
        );
    }

    #[motsu::test]
    fn revoking_nonexistent_role_member_has_no_effect(
        contract: Contract<AccessControlEnumerableExample>,
        admin: Address,
        alice: Address,
        charlie: Address,
    ) {
        // Grant [`AccessControl::DEFAULT_ADMIN_ROLE`] to `admin`.
        contract
            .sender(admin)
            .access_control
            ._grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), admin);

        contract
            .sender(admin)
            .grant_role(ROLE.into(), alice)
            .motsu_expect("should grant alice");

        // Try to revoke a role that `charlie` doesn't have.
        contract
            .sender(admin)
            .revoke_role(ROLE.into(), charlie)
            .motsu_expect("should not revert");

        // Check member count.
        assert_eq!(
            contract.sender(alice).get_role_member_count(ROLE.into()),
            U256::ONE
        );

        assert_eq!(
            contract
                .sender(alice)
                .get_role_member(ROLE.into(), U256::ZERO)
                .motsu_expect("should return alice"),
            alice
        );

        assert_eq!(
            contract.sender(alice).get_role_members(ROLE.into()),
            vec![alice]
        );
    }
}
