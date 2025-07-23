//! An upgradeability mechanism designed for UUPS proxies.
//!
//! The functions included here can perform an upgrade of an
//! [`Erc1967Proxy`], when this contract is set as the implementation
//! behind such a proxy.
//!
//! [`Erc1967Proxy`]: crate::proxy::erc1967::Erc1967Proxy
pub use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{abi::Bytes, contract, prelude::*, storage::StorageAddress};

/**

*
* A security mechanism ensures that an upgrade does not turn off
* upgradeability accidentally, although this risk is reinstated if the
* upgrade retains upgradeability but removes the security mechanism,
* e.g. by replacing `UUPSUpgradeable` with a custom implementation of
* upgrades.
*
* The {_authorizeUpgrade} function must be overridden to include
* access restriction to the upgrade mechanism.
*/
use super::IErc1822Proxiable;
use crate::proxy::erc1967::utils::{Erc1967Utils, IMPLEMENTATION_SLOT};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The call is from an unauthorized context.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error UUPSUnauthorizedCallContext();

        /// The storage `slot` is unsupported as a UUID.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error UUPSUnsupportedProxiableUUID(bytes32 slot);
    }
}

/// An [`UUPSUpgradeable`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The call is from an unauthorized context.
    UnauthorizedCallContext(UUPSUnauthorizedCallContext),
    /// The storage `slot` is unsupported as a UUID.
    UnsupportedProxiableUUID(UUPSUnsupportedProxiableUUID),
}

/// TODO
#[interface_id]
pub trait IUUPSUpgradeable: IErc1822Proxiable {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// The version of the upgrade interface of the contract. If this getter is
    /// missing, both `upgradeTo(address)` and `upgradeToAndCall(address,bytes)`
    /// are present, and `upgradeTo` must be used if no function should be
    /// called, while `upgradeToAndCall` will invoke the `receive` function if
    /// the second argument is the empty byte string. If the getter returns
    /// `"5.0.0"`, only `upgradeToAndCall(address,bytes)` is present, and the
    /// second argument must be the empty byte string if no function should be
    /// called, making it impossible to invoke the `receive` function during an
    /// upgrade.

    // const UPGRADE_INTERFACE_VERSION: &'static str = "5.0.0";

    /// Upgrade the implementation of the proxy to `newImplementation`, and
    /// subsequently execute the function call encoded in `data`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - TODO.
    /// * `new_implementation` - TODO.
    /// * `data` - TODO.
    ///
    /// # Errors
    ///
    /// TODO!
    ///
    /// # Events
    ///
    /// * [`crate::proxy::erc1967::Upgraded`]: Emitted when the implementation
    ///   is upgraded.
    #[selector(name = "upgradeToAndCall")]
    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Self::Error>;
}

#[storage]
/// TODO
pub struct UUPSUpgradeable {
    self_address: StorageAddress,
}

#[public]
#[implements(IUUPSUpgradeable<Error = Error>, IErc1822Proxiable)]
impl UUPSUpgradeable {
    #[constructor]
    fn constructor(&mut self) {
        self.self_address.set(contract::address());
    }
}

#[public]
impl IUUPSUpgradeable for UUPSUpgradeable {
    type Error = Error;

    #[selector(name = "upgradeToAndCall")]
    #[payable]
    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Error> {
        self.only_proxy()?;
        self._authorize_upgrade(new_implementation)?;
        self._upgrade_to_and_call_uups(new_implementation, &data)
    }
}

impl UUPSUpgradeable {
    /// Check that the execution is being performed through a `delegatecall`
    /// call and that the execution context is a proxy contract with an
    /// implementation (as defined in ERC-1967) pointing to [`self`]. This
    /// should only be the case for UUPS and transparent proxies that are using
    /// the current contract as their implementation. Execution of a function
    /// through ERC-1167 minimal proxies (clones) would not normally pass this
    /// test, but is not guaranteed to fail.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// TODO!
    pub fn only_proxy(&self) -> Result<(), Error> {
        self._check_proxy()
    }

    /// Check that the execution is not being performed through a delegate call.
    /// This allows a function to be callable on the implementing contract
    /// but not through proxies.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// TODO!
    pub fn not_delegated(&self) -> Result<(), Error> {
        self._check_not_delegated()
    }
}

#[public]
impl IErc1822Proxiable for UUPSUpgradeable {
    fn proxiable_uuid(&self) -> Result<U256, Vec<u8>> {
        self.not_delegated()?;
        Ok(IMPLEMENTATION_SLOT)
    }
}

impl UUPSUpgradeable {
    /// Reverts if the execution is performed via delegatecall.
    ///
    /// See [`Self::not_delegated`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`]: TODO!
    fn _check_not_delegated(&self) -> Result<(), Error> {
        if contract::address() != self.self_address.get() {
            Err(Error::UnauthorizedCallContext(UUPSUnauthorizedCallContext {}))
        } else {
            Ok(())
        }
    }

    /// Reverts if the execution is not performed via delegatecall or the
    /// execution context is not of a proxy with an ERC-1967 compliant
    /// implementation pointing to self.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UUPSUnauthorizedCallContext`]: TODO!
    fn _check_proxy(&self) -> Result<(), Error> {
        let self_address = self.self_address.get();
        if contract::address() == self_address
            || Erc1967Utils::get_implementation() != self_address
        {
            Err(Error::UnauthorizedCallContext(UUPSUnauthorizedCallContext {}))
        } else {
            Ok(())
        }
    }

    /// Function that should revert when [`stylus_sdk::msg::sender`] is not
    /// authorized to upgrade the contract. Called by
    /// [`Self::upgrade_to_and_call`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `_new_implementation` - The address of the new implementation.
    ///
    /// # Errors
    ///
    /// TODO!
    fn _authorize_upgrade(
        &self,
        _new_implementation: Address,
    ) -> Result<(), Error> {
        todo!()
    }

    /// Performs an implementation upgrade with a security check for UUPS
    /// proxies, and additional setup call.
    ///
    /// As a security check, [`IErc1822Proxiable::proxiable_uuid`] is invoked
    /// in the new implementation, and the return value is expected to be the
    /// implementation slot in ERC-1967.
    ///
    /// # Events
    ///
    /// * [`crate::proxy::erc1967::Erc1967Proxy::Upgraded`]: Emitted when the
    ///   implementation is upgraded.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `_new_implementation` - TODO
    /// * `_data` - TODO
    ///
    /// # Errors
    ///
    /// TODO!
    fn _upgrade_to_and_call_uups(
        &mut self,
        _new_implementation: Address,
        _data: &Bytes,
    ) -> Result<(), Error> {
        // try IERC1822Proxiable(newImplementation).proxiableUUID() returns
        // (bytes32 slot) {     if (slot !=
        // ERC1967Utils.IMPLEMENTATION_SLOT) {         revert
        // UUPSUnsupportedProxiableUUID(slot);     }
        //     ERC1967Utils.upgradeToAndCall(newImplementation, data);
        // } catch {
        //     // The implementation is not UUPS
        //     revert
        // ERC1967Utils.ERC1967InvalidImplementation(newImplementation);
        // }
        todo!()
    }
}
