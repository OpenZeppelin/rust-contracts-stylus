//! An upgradeability mechanism designed for UUPS (Universal Upgradeable Proxy
//! Standard) proxies as defined in [ERC-1822].
//!
//! [ERC-1822]: https://eips.ethereum.org/EIPS/eip-1822
//!
//! The functions included here can perform an upgrade of an
//! [`Erc1967Proxy`], when this contract is set as the implementation
//! behind such a proxy.
//!
//! [`Erc1967Proxy`]: crate::proxy::erc1967::Erc1967Proxy
pub use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::{aliases::B256, Address};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    call::{Call, MethodError},
    contract,
    prelude::*,
    storage::StorageAddress,
};

use crate::{
    proxy::{
        erc1967::{
            self,
            utils::{
                ERC1967InvalidAdmin, ERC1967InvalidBeacon,
                ERC1967InvalidImplementation, ERC1967NonPayable, Erc1967Utils,
                IMPLEMENTATION_SLOT,
            },
        },
        utils::erc1822::{Erc1822ProxiableInterface, IErc1822Proxiable},
    },
    utils::address,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The call is from an unauthorized context.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error UUPSUnauthorizedCallContext();

        /// The storage `slot` is unsupported as a UUID.
        /// * `slot` - The unsupported UUID returned by the implementation.
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
    /// Indicates an error related to the fact that the `implementation`
    /// of the proxy is invalid.
    InvalidImplementation(ERC1967InvalidImplementation),
    /// Indicates an error related to the fact that the `admin` of the
    /// proxy is invalid.
    InvalidAdmin(ERC1967InvalidAdmin),
    /// Indicates an error related to the fact that the `beacon`
    /// of the proxy is invalid.
    InvalidBeacon(ERC1967InvalidBeacon),
    /// Indicates an error related to the fact that an upgrade function
    /// sees [`stylus_sdk::msg::value()`] > [`alloy_primitives::U256::ZERO`]
    /// that may be lost.
    NonPayable(ERC1967NonPayable),
    /// There's no code at `target` (it is not a contract).
    EmptyCode(address::AddressEmptyCode),
    /// A call to an address target failed. The target may have reverted.
    FailedCall(address::FailedCall),
    /// Indicates an error related to the fact that the delegate call
    /// failed.
    FailedCallWithReason(address::FailedCallWithReason),
}

impl From<erc1967::utils::Error> for Error {
    fn from(e: erc1967::utils::Error) -> Self {
        match e {
            erc1967::utils::Error::InvalidImplementation(e) => {
                Error::InvalidImplementation(e)
            }
            erc1967::utils::Error::InvalidAdmin(e) => Error::InvalidAdmin(e),
            erc1967::utils::Error::InvalidBeacon(e) => Error::InvalidBeacon(e),
            erc1967::utils::Error::NonPayable(e) => Error::NonPayable(e),
            erc1967::utils::Error::EmptyCode(e) => Error::EmptyCode(e),
            erc1967::utils::Error::FailedCall(e) => Error::FailedCall(e),
            erc1967::utils::Error::FailedCallWithReason(e) => {
                Error::FailedCallWithReason(e)
            }
        }
    }
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for UUPSUpgradeable {}

/// Interface for a UUPS (Universal Upgradeable Proxy Standard) upgradeable
/// contract.
#[interface_id]
pub trait IUUPSUpgradeable: IErc1822Proxiable {
    /// The error type associated to the trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// The version of the upgrade interface of the contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "UPGRADE_INTERFACE_VERSION")]
    fn upgrade_interface_version(&self) -> String {
        String::from("5.0.0")
    }

    /// Upgrade the implementation of the proxy to `new_implementation`, and
    /// subsequently execute the function call encoded in `data`.
    ///
    /// Note: This function should revert when [`stylus_sdk::msg::sender`] is
    /// not authorized to upgrade the contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation contract.
    /// * `data` - Additional data to be passed to the new implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the call is not made through a
    ///   valid proxy context.
    /// * [`Error::InvalidImplementation`] - If the new implementation address
    ///   is invalid or doesn't implement the required interface.
    /// * [`Error::UnsupportedProxiableUUID`] - If the new implementation
    ///   returns an unsupported UUID.
    /// * [`Error::NonPayable`] - If the upgrade function receives ETH but is
    ///   not designed to handle it.
    /// * [`Error::EmptyCode`] - If there's no code at the new implementation
    ///   address.
    /// * [`Error::FailedCall`] - If the delegate call to the new implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the delegate call fails with a
    ///   specific reason.
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

/// State of a [`UUPSUpgradeable`] contract.
#[storage]
pub struct UUPSUpgradeable {
    /// The address of this contract, used for context validation.
    self_address: StorageAddress,
}

#[public]
#[implements(IUUPSUpgradeable<Error = Error>, IErc1822Proxiable)]
impl UUPSUpgradeable {
    /// Initializes the contract by storing its own address for later context
    /// validation.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
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
    ) -> Result<(), Self::Error> {
        self.only_proxy()?;
        self._upgrade_to_and_call_uups(new_implementation, &data)
    }
}

impl UUPSUpgradeable {
    /// Check that the execution is being performed through a
    /// [`stylus_sdk::call::delegate_call`] call and that the execution
    /// context is a proxy contract with an implementation (as defined in
    /// ERC-1967) pointing to `self`. This should only be the case for
    /// UUPS and transparent proxies that are using the current contract as
    /// their implementation. Execution of a function through ERC-1167
    /// minimal proxies (clones) would not normally pass this test, but is
    /// not guaranteed to fail.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is not performed
    ///   through a delegate call or the execution context is not of a proxy
    ///   with an ERC-1967 compliant implementation pointing to self.
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
    /// * [`Error::UnauthorizedCallContext`] - If the execution is performed via
    ///   delegate call.
    pub fn not_delegated(&self) -> Result<(), Error> {
        self._check_not_delegated()
    }
}

#[public]
impl IErc1822Proxiable for UUPSUpgradeable {
    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        self.not_delegated()?;
        Ok(IMPLEMENTATION_SLOT)
    }
}

impl UUPSUpgradeable {
    /// Reverts if the execution is performed via delegate call.
    ///
    /// See [`Self::not_delegated`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is performed via
    ///   delegate call.
    fn _check_not_delegated(&self) -> Result<(), Error> {
        if contract::address() == self.self_address.get() {
            Ok(())
        } else {
            Err(Error::UnauthorizedCallContext(UUPSUnauthorizedCallContext {}))
        }
    }

    /// Reverts if the execution is not performed via delegate call or the
    /// execution context is not of a proxy with an ERC-1967 compliant
    /// implementation pointing to self.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::UnauthorizedCallContext`] - If the execution is not performed
    ///   via delegate call or the execution context is not of a proxy with an
    ///   ERC-1967 compliant implementation pointing to self.
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

    /// Performs an implementation upgrade with a security check for UUPS
    /// proxies, and additional setup call.
    ///
    /// As a security check, [`IErc1822Proxiable::proxiable_uuid`] is invoked
    /// in the new implementation, and the return value is expected to be the
    /// implementation slot in ERC-1967.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_implementation` - The address of the new implementation.
    /// * `data` - The data to pass to the new implementation.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If the new implementation doesn't
    ///   support the required interface or is invalid.
    /// * [`Error::UnsupportedProxiableUUID`] - If the new implementation
    ///   returns an unsupported UUID.
    /// * [`Error::NonPayable`] - If the upgrade function receives ETH but is
    ///   not designed to handle it.
    /// * [`Error::EmptyCode`] - If there's no code at the new implementation
    ///   address.
    /// * [`Error::FailedCall`] - If the delegate call to the new implementation
    ///   fails.
    /// * [`Error::FailedCallWithReason`] - If the delegate call fails with a
    ///   specific reason.
    ///
    /// # Events
    ///
    /// * [`crate::proxy::erc1967::Erc1967Proxy::Upgraded`]: Emitted when the
    ///   implementation is upgraded.
    fn _upgrade_to_and_call_uups(
        &mut self,
        new_implementation: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        let slot = Erc1822ProxiableInterface::new(new_implementation)
            .proxiable_uuid(Call::new_in(self))
            .map_err(|_e| {
                Error::InvalidImplementation(ERC1967InvalidImplementation {
                    implementation: new_implementation,
                })
            })?;

        if slot == IMPLEMENTATION_SLOT {
            Erc1967Utils::upgrade_to_and_call(self, new_implementation, data)
                .map_err(Error::from)
        } else {
            Err(Error::UnsupportedProxiableUUID(UUPSUnsupportedProxiableUUID {
                slot,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};
    use example_uups::*;
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::*;
    #[cfg_attr(coverage_nightly, coverage(off))]
    mod example_uups {
        use stylus_sdk::storage::StorageU256;

        use super::*;
        use crate::access::ownable::IOwnable;

        pub trait IExampleUups: IUUPSUpgradeable + IOwnable {
            fn set_value(&mut self, value: U256);
            fn get_value(&self) -> U256;
            fn version(&self) -> String;
        }

        #[storage]
        pub struct ExampleUUPSv1 {
            value: StorageU256,
        }

        #[storage]
        pub struct ExampleUUPSv2 {
            value: StorageU256,
        }

        #[public]
        #[implements(IExampleUups<Error = Error>, IOwnable)]
        impl ExampleUUPSv1 {
            #[constructor]
            fn constructor(&mut self, owner: Address) {
                self.ownable.constructor(owner);
                self.value.set(U256::from(1));
            }

            fn set_value(&mut self, value: U256) -> Result<(), Error> {
                self.value.set(value);
                Ok(())
            }

            fn get_value(&self) -> U256 {
                self.value.get()
            }

            fn version(&self) -> String {
                String::from("1.0.0")
            }
        }

        #[public]
        impl IUUPSUpgradeable for ExampleUUPSv1 {
            fn upgrade_to_and_call(
                &mut self,
                new_implementation: Address,
                data: Bytes,
            ) -> Result<(), Error> {
                self.upgrade_to(new_implementation)?;
                self.upgrade_to_and_call(new_implementation, data)
            }
        }

        #[public]
        impl IOwnable for ExampleUUPSv1 {
            fn owner(&self) -> Address {
                self.ownable.owner()
            }
        }
    }
}
