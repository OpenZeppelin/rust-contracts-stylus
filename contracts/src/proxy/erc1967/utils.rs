//! This library provides getters and event emitting update functions for
//! [ERC-1967] slots.
//!
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967

use alloy_primitives::{uint, Address, U256};
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    call::{MethodError, RawCall},
    evm, msg,
    prelude::*,
    storage::StorageAddress,
};

use crate::{
    proxy::{beacon::IBeaconInterface, erc1967},
    utils::storage_slot::StorageSlot,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates an error related to the fact that the `implementation`
        /// of the proxy is invalid.
        ///
        /// * `implementation` - Address of the invalid implementation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967InvalidImplementation(address implementation);

        /// Indicates an error related to the fact that the `admin` of the
        /// proxy is invalid.
        ///
        /// * `admin` - Address of the invalid admin.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967InvalidAdmin(address admin);

        /// Indicates an error related to the fact that the `beacon`
        /// of the proxy is invalid.
        ///
        /// * `beacon` - Address of the invalid `beacon` of the proxy.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967InvalidBeacon(address beacon);

        /// Indicates an error relatoed to the fact that an upgrade function
        /// sees [`stylus_sdk::msg::value()`] > [`U256::ZERO`] that may be lost.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC1967NonPayable();
    }
}

/// An [`Erc1967Utils`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the fact that the `implementation`
    /// of the proxy is invalid.
    InvalidImplementation(ERC1967InvalidImplementation),
    /// Indicates an error related to the fact that the `admin` of the
    /// proxy is invalid.
    InvalidAdmin(ERC1967InvalidAdmin),
    /// Indicates an error related to the fact that the `beacon`
    /// of the proxy is invalid.
    InvalidBeacon(ERC1967InvalidBeacon),
    /// Indicates an error relatoed to the fact that an upgrade function
    /// sees [`stylus_sdk::msg::value()`] > [`alloy_primitives::U256::ZERO`]
    /// that may be lost.
    NonPayable(ERC1967NonPayable),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// Storage slot with the address of the current implementation.
/// This is the keccak-256 hash of "eip1967.proxy.implementation" subtracted by
/// 1.
const IMPLEMENTATION_SLOT: U256 = uint!(
    0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc_U256
);

/// Storage slot with the admin of the contract.
/// This is the keccak-256 hash of "eip1967.proxy.admin" subtracted by 1.
const ADMIN_SLOT: U256 = uint!(
    0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103_U256
);
/// The storage slot of the UpgradeableBeacon contract which defines the
/// implementation for this proxy.
///
/// This is the keccak-256 hash of "eip1967.proxy.beacon" subtracted by 1.
const BEACON_SLOT: U256 = uint!(
    0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50_U256
);

/// This library provides getters and event emitting update functions for
/// [ERC-1967] slots.
///
/// [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967
pub struct Erc1967Utils;

/// Implementation of the [`Erc1967Utils`] library.
impl Erc1967Utils {
    /// Returns the current implementation address.
    pub fn get_implementation() -> Address {
        StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).get()
    }

    /// Performs implementation upgrade with additional setup call if
    /// data is nonempty. This function is payable only if the setup call
    /// is performed, otherwise [`msg::value()`] is rejected to avoid stuck
    /// value in the contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Mutable access to the contract's state.
    /// * `new_implementation` - The new implementation address.
    /// * `data` - The data to pass to the setup call.
    ///
    /// # Errors (TODO)
    pub fn upgrade_to_and_call(
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::_set_implementation(new_implementation)?;

        evm::log(erc1967::Upgraded { implementation: new_implementation });

        if data.len() > 0 {
            // TODO: extract to Address library
            unsafe {
                RawCall::new_delegate()
                    .flush_storage_cache()
                    .call(new_implementation, data.as_slice())
                    .expect("TODO: handle error");
            }
        } else {
            Erc1967Utils::_check_non_payable()?;
        }

        Ok(())
    }

    /// Returns the current admin.
    pub fn get_admin() -> Address {
        StorageSlot::get_slot::<StorageAddress>(ADMIN_SLOT).get()
    }

    /// Changes the admin of the proxy.
    ///
    /// # Arguments
    ///
    /// * `new_admin` - The new admin address.
    pub fn change_admin(new_admin: Address) -> Result<(), Error> {
        evm::log(erc1967::AdminChanged {
            previous_admin: Erc1967Utils::get_admin(),
            new_admin,
        });

        Erc1967Utils::_set_admin(new_admin)
    }

    /// Returns the current beacon.
    pub fn get_beacon() -> Address {
        StorageSlot::get_slot::<StorageAddress>(BEACON_SLOT).get()
    }

    /// Change the beacon and trigger a setup call if data is nonempty.
    /// This function is payable only if the setup call is performed,
    /// otherwise [`msg::value()`] is rejected to avoid stuck value in the
    /// contract.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Mutable access to the contract's state.
    /// * `new_beacon` - The new beacon address.
    /// * `data` - The data to pass to the setup call.
    pub fn upgrade_beacon_to_and_call<T: TopLevelStorage>(
        context: &T,
        new_beacon: Address,
        data: Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::_set_beacon(context, new_beacon)?;
        evm::log(erc1967::BeaconUpgraded { beacon: new_beacon });

        if data.len() > 0 {
            let beacon_implementation =
                Erc1967Utils::get_beacon_implementation(context, new_beacon)?;

            // TODO: extract to Address library
            unsafe {
                RawCall::new_delegate()
                    .flush_storage_cache()
                    .call(beacon_implementation, data.as_slice())
                    .expect("TODO: handle error");
            }
        } else {
            Erc1967Utils::_check_non_payable()?;
        }

        Ok(())
    }
}

impl Erc1967Utils {
    /// Reverts if [`msg::value()`] is not [`alloy_primitives::U256::ZERO`]. It
    /// can be used to avoid [`msg::value()`] stuck in the contract if an
    /// upgrade does not perform an initialization call.
    ///
    /// # Errors
    ///
    /// * [`Error::NonPayable`] - If [`msg::value()`] is not
    ///   [`alloy_primitives::U256::ZERO`].
    fn _check_non_payable() -> Result<(), Error> {
        if msg::value().is_zero() {
            Ok(())
        } else {
            Err(ERC1967NonPayable {}.into())
        }
    }

    /// Stores a new address in the ERC-1967 implementation slot.
    ///
    /// # Arguments
    ///
    /// * `new_implementation` - The new implementation address.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidImplementation`] - If the `new_implementation` address
    ///   is not a valid implementation.
    fn _set_implementation(new_implementation: Address) -> Result<(), Error> {
        if !new_implementation.has_code() {
            return Err(ERC1967InvalidImplementation {
                implementation: new_implementation,
            }
            .into());
        }

        StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT)
            .set(new_implementation);

        Ok(())
    }

    /// Stores a new address in the ERC-1967 admin slot.
    ///
    /// # Arguments
    ///
    /// * `new_admin` - The new admin address.
    ///
    /// # Errors (TODO)
    fn _set_admin(new_admin: Address) -> Result<(), Error> {
        if new_admin.is_zero() {
            return Err(ERC1967InvalidAdmin { admin: new_admin }.into());
        }

        StorageSlot::get_slot::<StorageAddress>(ADMIN_SLOT).set(new_admin);

        Ok(())
    }

    /// Stores a new beacon in the ERC-1967 beacon slot.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Mutable access to the contract's state.
    /// * `new_beacon` - The new beacon address.
    ///
    /// # Errors (TODO)
    fn _set_beacon<T: TopLevelStorage>(
        context: &T,
        new_beacon: Address,
    ) -> Result<(), Error> {
        if !new_beacon.has_code() {
            return Err(ERC1967InvalidBeacon { beacon: new_beacon }.into());
        }

        StorageSlot::get_slot::<StorageAddress>(BEACON_SLOT).set(new_beacon);

        let beacon_implementation =
            Erc1967Utils::get_beacon_implementation(context, new_beacon)?;

        if !beacon_implementation.has_code() {
            return Err(ERC1967InvalidImplementation {
                implementation: beacon_implementation,
            }
            .into());
        }

        Ok(())
    }
}

impl Erc1967Utils {
    fn get_beacon_implementation<T: TopLevelStorage>(
        context: &T,
        beacon: Address,
    ) -> Result<Address, Error> {
        IBeaconInterface::new(beacon).implementation(context).map_err(|e| {
            panic!("TODO: handle error: {:?}", e);
        })
    }
}
