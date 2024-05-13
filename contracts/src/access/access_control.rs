//! Contract module that allows children to implement role-based access control
//! mechanisms. This is a lightweight version that doesn't allow enumerating
//! role members except through off-chain means by accessing the contract event
//! logs. Some applications may benefit from on-chain enumerability, for those
//! cases see [`AccessControlEnumberable`][enumerable ext].
//!
//! Roles are referred to by their `bytes32` identifier. These should be exposed
//! in the external API and be unique. The best way to achieve this is by using
//! `public constant` hash digests:
//!
//! TODO: Update example to Rust.
//!
//! ```solidity
//! bytes32 public constant MY_ROLE = keccak256("MY_ROLE");
//! ```
//!
//! Roles can be used to represent a set of permissions. To restrict access to a
//! function call, use `has_role`:
//!
//! TODO: Update example to Rust.
//!
//! ```solidity
//! function foo() public {
//!     require(hasRole(MY_ROLE, msg.sender));
//!     ...
//! }
//! ```
//!
//! Roles can be granted and revoked dynamically via the `grant_role` and
//! `revoke_role` functions. Each role has an associated admin role, and only
//! accounts that have a role's admin role can call `grant_role` and
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
//!
//! [enumerable ext]: TBD
use alloy_primitives::Address;
use alloy_sol_types::sol;
use stylus_proc::SolidityError;
use stylus_sdk::{
    evm, msg,
    stylus_proc::{external, sol_storage},
};

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
    /// [`AccessControl::grantRole`].
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
    #[derive(Debug)]
    #[allow(missing_docs)]
    error AccessControlUnauthorizedAccount(address account, bytes32 neededRole);
    /// The caller of a function is not the expected one.
    ///
    /// NOTE: Don't confuse with [`AccessControlUnauthorizedAccount`].
    #[derive(Debug)]
    #[allow(missing_docs)]
    error AccessControlBadConfirmation();
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
