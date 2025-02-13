//! Contract module that allows children to implement role-based access control
//! mechanisms.
//!
//! This is a lightweight version that doesn't allow enumerating role members
//! except through off-chain means by accessing the contract event logs.
//!
//! Roles are referred to by their `bytes32` identifier. These should be exposed
//! in the external API and be unique. The best way to achieve this is by using
//! `pub const` hash digests:
//!
//! ```no_run
//! pub const MY_ROLE: [u8; 32] =
//!     keccak_const::Keccak256::new().update(b"MY_ROLE").finalize();
//! ```
//!
//! Roles can be used to represent a set of permissions. To restrict access to a
//! function call, use [`AccessControl::has_role`]:
//!
//! ```rust,ignore
//! pub fn foo() {
//!   assert!(self.has_role(MY_ROLE.into(), msg::sender()));
//!   // ...
//! }
//! ```
//!
//! Roles can be granted and revoked dynamically via the `grant_role` and
//! `revoke_role` functions. Each role has an associated admin role, and only
//! accounts that have a `role`'s `admin_role` can call `grant_role` and
//! `revoke_role`.
//!
//! By default, the admin role for all roles is `DEFAULT_ADMIN_ROLE`, which
//! means that only accounts with this role will be able to grant or revoke
//! other roles. More complex role relationships can be created by using
//! `_set_role_admin`.
//!
//! WARNING: The `DEFAULT_ADMIN_ROLE` is also its own admin: it has permission
//! to grant and revoke this role. Extra precautions should be taken to secure
//! accounts that have been granted it. We recommend using
//! `AccessControlDefaultAdminRules` to enforce additional security measures for
//! this role.
use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, B256};
pub use sol::*;
use stylus_sdk::{
    evm, msg,
    prelude::storage,
    storage::{StorageBool, StorageFixedBytes, StorageMap},
    stylus_proc::{public, SolidityError},
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when `new_admin_role` is set as `role`'s admin role, replacing
        /// `previous_admin_role`.
        ///
        /// `DEFAULT_ADMIN_ROLE` is the starting admin for all roles, despite
        /// `RoleAdminChanged` not being emitted signaling this.
        #[allow(missing_docs)]
        event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previous_admin_role, bytes32 indexed new_admin_role);
        /// Emitted when `account` is granted `role`.
        ///
        /// `sender` is the account that originated the contract call. This account
        /// bears the admin role (for the granted role).
        /// Expected in cases where the role was granted using the internal
        /// [`AccessControl::grant_role`].
        #[allow(missing_docs)]
        event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender);
        /// Emitted when `account` is revoked `role`.
        ///
        /// `sender` is the account that originated the contract call:
        ///   - if using `revoke_role`, it is the admin role bearer.
        ///   - if using `renounce_role`, it is the role bearer (i.e. `account`).
        #[allow(missing_docs)]
        event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender);
    }

    sol! {
        /// The `account` is missing a role.
        ///
        /// * `account` - Account that was found to not be authorized.
        /// * `needed_role` - The missing role.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error AccessControlUnauthorizedAccount(address account, bytes32 needed_role);
        /// The caller of a function is not the expected one.
        ///
        /// NOTE: Don't confuse with [`AccessControlUnauthorizedAccount`].
        #[derive(Debug)]
        #[allow(missing_docs)]
        error AccessControlBadConfirmation();
    }
}

/// An error that occurred in the implementation of an [`AccessControl`]
/// contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The caller account is missing a role.
    UnauthorizedAccount(AccessControlUnauthorizedAccount),
    /// The caller of a afunction is not the expected one.
    BadConfirmation(AccessControlBadConfirmation),
}

/// State of a [`RoleData`] contract.
///
/// Stores information about a specific role.
#[storage]
pub struct RoleData {
    /// Whether an account is member of a certain role.
    pub has_role: StorageMap<Address, StorageBool>,
    /// The admin role for this role.
    pub admin_role: StorageFixedBytes<32>,
}

/// State of an [`AccessControl`] contract.
#[storage]
pub struct AccessControl {
    /// Role identifier -> Role information.
    #[allow(clippy::used_underscore_binding)]
    pub _roles: StorageMap<FixedBytes<32>, RoleData>,
}

#[public]
impl AccessControl {
    /// Returns `true` if `account` has been granted `role`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    /// * `account` - The account to check for membership.
    #[must_use]
    pub fn has_role(&self, role: B256, account: Address) -> bool {
        self._roles.getter(role).has_role.get(account)
    }

    /// Checks if [`msg::sender`] has been granted `role`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedAccount`] - If [`msg::sender`] has not been
    ///   granted `role`.
    pub fn only_role(&self, role: B256) -> Result<(), Error> {
        self._check_role(role, msg::sender())
    }

    /// Returns the admin role that controls `role`. See [`Self::grant_role`]
    /// and [`Self::revoke_role`].
    ///
    /// To change a role's admin, use [`Self::_set_role_admin`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    #[must_use]
    pub fn get_role_admin(&self, role: B256) -> B256 {
        *self._roles.getter(role).admin_role
    }

    /// Grants `role` to `account`.
    ///
    /// If `account` had not been already granted `role`, emits a
    /// [`RoleGranted`] event.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `role` - The role identifier.
    /// * `account` - The account which will be granted the role.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedAccount`] - If [`msg::sender`] has not been
    ///   granted `role`.
    ///
    /// # Events
    ///
    /// * [`RoleGranted`]
    pub fn grant_role(
        &mut self,
        role: B256,
        account: Address,
    ) -> Result<(), Error> {
        let admin_role = self.get_role_admin(role);
        self.only_role(admin_role)?;
        self._grant_role(role, account);
        Ok(())
    }

    /// Revokes `role` from `account`.
    ///
    /// If `account` had been granted `role`, emits a [`RoleRevoked`] event.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `role` - The role identifier.
    /// * `account` - The account which will be revoked the role.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedAccount`] - If [`msg::sender`] has not been
    ///   granted `role`.
    ///
    /// # Events
    ///
    /// * [`RoleRevoked`].
    pub fn revoke_role(
        &mut self,
        role: B256,
        account: Address,
    ) -> Result<(), Error> {
        let admin_role = self.get_role_admin(role);
        self.only_role(admin_role)?;
        self._revoke_role(role, account);
        Ok(())
    }

    /// Revokes `role` from the calling account.
    ///
    /// Roles are often managed via [`Self::grant_role`] and
    /// [`Self::revoke_role`]: this function's purpose is to provide a mechanism
    /// for accounts to lose their privileges if they are compromised (such as
    /// when a trusted device is misplaced).
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `role` - The role identifier.
    /// * `confirmation` - The account which will be revoked the role.
    ///
    /// # Errors
    ///
    /// * [`Error::BadConfirmation`]  - If [`msg::sender`] is not the
    ///   `confirmation` address.
    ///
    /// # Events
    ///
    /// * [`RoleRevoked`] - If the calling account has its `role` revoked.
    pub fn renounce_role(
        &mut self,
        role: B256,
        confirmation: Address,
    ) -> Result<(), Error> {
        if msg::sender() != confirmation {
            return Err(Error::BadConfirmation(
                AccessControlBadConfirmation {},
            ));
        }

        self._revoke_role(role, confirmation);
        Ok(())
    }
}

impl AccessControl {
    /// The default admin role. `[0; 32]` by default.
    pub const DEFAULT_ADMIN_ROLE: [u8; 32] = [0; 32];

    /// Sets `admin_role` as `role`'s admin role.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `role` - The identifier of the role we are changing the admin to.
    /// * `new_admin_role` - The new admin role.
    ///
    /// # Events
    ///
    /// * [`RoleAdminChanged`].
    pub fn _set_role_admin(&mut self, role: B256, new_admin_role: B256) {
        let previous_admin_role = self.get_role_admin(role);
        self._roles.setter(role).admin_role.set(new_admin_role);
        evm::log(RoleAdminChanged {
            role,
            previous_admin_role,
            new_admin_role,
        });
    }

    /// Checks if `account` has been granted `role`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `role` - The role identifier.
    /// * `account` - The account to check for membership.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedAccount`] - If [`msg::sender`] has not been
    ///   granted `role`.
    pub fn _check_role(
        &self,
        role: B256,
        account: Address,
    ) -> Result<(), Error> {
        if !self.has_role(role, account) {
            return Err(Error::UnauthorizedAccount(
                AccessControlUnauthorizedAccount { account, needed_role: role },
            ));
        }

        Ok(())
    }

    /// Attempts to grant `role` to `account` and returns a boolean indicating
    /// if `role` was granted.
    ///
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `role` - The role identifier.
    /// * `account` - The account which will be granted the role.
    ///
    /// # Events
    ///
    /// * [`RoleGranted`].
    pub fn _grant_role(&mut self, role: B256, account: Address) -> bool {
        if self.has_role(role, account) {
            false
        } else {
            self._roles.setter(role).has_role.insert(account, true);
            evm::log(RoleGranted { role, account, sender: msg::sender() });
            true
        }
    }

    /// Attempts to revoke `role` from `account` and returns a boolean
    /// indicating if `role` was revoked.
    ///
    /// Internal function without access restriction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `role` - The role identifier.
    /// * `account` - The account which will be granted the role.
    ///
    /// # Events
    ///
    /// * [`RoleRevoked`].
    pub fn _revoke_role(&mut self, role: B256, account: Address) -> bool {
        if self.has_role(role, account) {
            self._roles.setter(role).has_role.insert(account, false);
            evm::log(RoleRevoked { role, account, sender: msg::sender() });
            true
        } else {
            false
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address};
    use stylus_sdk::msg;

    use super::{AccessControl, Error};

    /// Shorthand for declaring variables converted from a hex literal to a
    /// fixed 32-byte slice;
    macro_rules! roles {
        ($($var:ident = $hex:literal);* $(;)?) => {
            $(
                const $var: [u8; 32] = alloy_primitives::hex!($hex);
            )*
        };
    }

    roles! {
        ROLE       = "ed9ea7bc2a13bc59432ab07436e7f7f5450f82d4b48c401bed177bfaf36b1873";
        OTHER_ROLE = "879ce0d4bfd332649ca3552efe772a38d64a315eb70ab69689fd309c735946b5";
    }

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    // Since we don't have constructors, we need to call this  to setup
    // `msg::sender` as a member of `ROLE`.
    //
    // NOTE: Once we have support for setting `msg::sender` and constructor,
    // this function shouldn't be needed.
    fn _grant_role_to_msg_sender(contract: &mut AccessControl, role: [u8; 32]) {
        contract
            ._roles
            .setter(role.into())
            .has_role
            .insert(msg::sender(), true);
    }

    #[motsu::test]
    fn default_role_is_default_admin(contract: AccessControl) {
        let role_admin = contract.get_role_admin(ROLE.into());
        assert_eq!(role_admin, AccessControl::DEFAULT_ADMIN_ROLE);
    }

    #[motsu::test]
    fn default_admin_roles_admin_is_itself(contract: AccessControl) {
        const DEFAULT_ADMIN_ROLE: [u8; 32] = AccessControl::DEFAULT_ADMIN_ROLE;
        let role_admin = contract.get_role_admin(DEFAULT_ADMIN_ROLE.into());
        assert_eq!(role_admin, DEFAULT_ADMIN_ROLE);
    }

    #[motsu::test]
    fn non_admin_cannot_grant_role_to_others(contract: AccessControl) {
        let err = contract.grant_role(ROLE.into(), ALICE).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn accounts_can_be_granted_roles_multiple_times(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, AccessControl::DEFAULT_ADMIN_ROLE);

        contract.grant_role(ROLE.into(), ALICE).unwrap();
        contract.grant_role(ROLE.into(), ALICE).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(has_role);
    }

    #[motsu::test]
    fn not_granted_roles_can_be_revoked(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, AccessControl::DEFAULT_ADMIN_ROLE);

        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(!has_role);
        contract.revoke_role(ROLE.into(), ALICE).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(!has_role);
    }

    #[motsu::test]
    fn admin_can_revoke_role(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, AccessControl::DEFAULT_ADMIN_ROLE);
        contract._roles.setter(ROLE.into()).has_role.insert(ALICE, true);

        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(has_role);
        contract.revoke_role(ROLE.into(), ALICE).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(!has_role);
    }

    #[motsu::test]
    fn non_admin_cannot_revoke_role(contract: AccessControl) {
        contract._roles.setter(ROLE.into()).has_role.insert(ALICE, true);

        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(has_role);
        let err = contract.revoke_role(ROLE.into(), ALICE).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn roles_can_be_revoked_multiple_times(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, AccessControl::DEFAULT_ADMIN_ROLE);

        contract.revoke_role(ROLE.into(), ALICE).unwrap();
        contract.revoke_role(ROLE.into(), ALICE).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(!has_role);
    }

    #[motsu::test]
    fn bearer_can_renounce_role(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, ROLE);

        let has_role = contract.has_role(ROLE.into(), msg::sender());
        assert!(has_role);
        contract.renounce_role(ROLE.into(), msg::sender()).unwrap();
        let has_role = contract.has_role(ROLE.into(), msg::sender());
        assert!(!has_role);
    }

    #[motsu::test]
    fn only_sender_can_renounce(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, ROLE);
        let err = contract.renounce_role(ROLE.into(), ALICE).unwrap_err();
        assert!(matches!(err, Error::BadConfirmation(_)));
    }

    #[motsu::test]
    fn roles_can_be_renounced_multiple_times(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, ROLE);

        let sender = msg::sender();
        contract.renounce_role(ROLE.into(), sender).unwrap();
        contract.renounce_role(ROLE.into(), sender).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(!has_role);
    }

    #[motsu::test]
    fn a_roles_admin_role_can_change(contract: AccessControl) {
        contract._set_role_admin(ROLE.into(), OTHER_ROLE.into());
        _grant_role_to_msg_sender(contract, OTHER_ROLE);

        let admin_role = contract.get_role_admin(ROLE.into());
        assert_eq!(admin_role, OTHER_ROLE);
    }

    #[motsu::test]
    fn the_new_admin_can_grant_roles(contract: AccessControl) {
        contract._set_role_admin(ROLE.into(), OTHER_ROLE.into());
        _grant_role_to_msg_sender(contract, OTHER_ROLE);

        contract.grant_role(ROLE.into(), ALICE).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(has_role);
    }

    #[motsu::test]
    fn the_new_admin_can_revoke_roles(contract: AccessControl) {
        contract._set_role_admin(ROLE.into(), OTHER_ROLE.into());
        _grant_role_to_msg_sender(contract, OTHER_ROLE);

        contract._roles.setter(ROLE.into()).has_role.insert(ALICE, true);
        contract.revoke_role(ROLE.into(), ALICE).unwrap();
        let has_role = contract.has_role(ROLE.into(), ALICE);
        assert!(!has_role);
    }

    #[motsu::test]
    fn previous_admins_no_longer_grant_roles(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, ROLE);
        contract._set_role_admin(ROLE.into(), OTHER_ROLE.into());

        let err = contract.grant_role(ROLE.into(), ALICE).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn previous_admins_no_longer_revoke_roles(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, ROLE);
        contract._set_role_admin(ROLE.into(), OTHER_ROLE.into());

        let err = contract.revoke_role(ROLE.into(), ALICE).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn does_not_revert_if_sender_has_role(contract: AccessControl) {
        _grant_role_to_msg_sender(contract, ROLE);
        contract._check_role(ROLE.into(), msg::sender()).unwrap();
    }

    #[motsu::test]
    fn reverts_if_sender_doesnt_have_role(contract: AccessControl) {
        let err = contract._check_role(ROLE.into(), msg::sender()).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
        let err =
            contract._check_role(OTHER_ROLE.into(), msg::sender()).unwrap_err();
        assert!(matches!(err, Error::UnauthorizedAccount(_)));
    }

    #[motsu::test]
    fn internal_grant_role_true_if_no_role(contract: AccessControl) {
        let role_granted = contract._grant_role(ROLE.into(), ALICE);
        assert!(role_granted);
    }

    #[motsu::test]
    fn internal_grant_role_false_if_role(contract: AccessControl) {
        contract._roles.setter(ROLE.into()).has_role.insert(ALICE, true);
        let role_granted = contract._grant_role(ROLE.into(), ALICE);
        assert!(!role_granted);
    }

    #[motsu::test]
    fn internal_revoke_role_true_if_role(contract: AccessControl) {
        contract._roles.setter(ROLE.into()).has_role.insert(ALICE, true);
        let role_revoked = contract._revoke_role(ROLE.into(), ALICE);
        assert!(role_revoked);
    }

    #[motsu::test]
    fn internal_revoke_role_false_if_no_role(contract: AccessControl) {
        let role_revoked = contract._revoke_role(ROLE.into(), ALICE);
        assert!(!role_revoked);
    }
}
