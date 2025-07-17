//! This contract implements a proxy that gets the implementation address for
//! each call from an [UpgradeableBeacon][UpgradeableBeacon].
//!
//! The beacon address can only be set once during construction, and cannot be
//! changed afterwards. It is stored in an immutable variable to avoid
//! unnecessary storage reads, and also in the beacon storage slot specified by
//! [ERC-1967] so that it can be accessed externally.
//!
//! CAUTION: Since the beacon address can never be changed, you must ensure that
//! you either control the beacon, or trust the beacon to not upgrade the
//! implementation maliciously.
//!
//! IMPORTANT: Do not use the implementation logic to modify the beacon storage
//! slot. Doing so would leave the proxy in an inconsistent state where the
//! beacon storage slot does not match the beacon address.
//!
//! [UpgradeableBeacon]: super::UpgradeableBeacon
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967

use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
use stylus_sdk::{abi::Bytes, prelude::*, storage::StorageAddress};

use crate::proxy::{
    beacon::IBeaconInterface,
    erc1967::{Erc1967Utils, Error},
    IProxy,
};

/// State of an [`BeaconProxy`] token.
#[storage]
pub struct BeaconProxy {
    beacon: StorageAddress,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for BeaconProxy {}

impl BeaconProxy {
    /// Initializes the proxy with `beacon`.
    ///
    /// If `data` is nonempty, it's used as data in a delegate call to the
    /// implementation returned by the beacon. This will typically be an
    /// encoded function call, and allows initializing the storage of the proxy
    /// like a Solidity constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `beacon` - The beacon address.
    /// * `data` - The data to pass to the beacon.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidBeacon`] - If the beacon is not a contract with the
    ///   interface [IBeacon][IBeacon].
    /// * [`Error::NonPayable`] - If the data is empty and
    ///   [msg::value][msg_value] is not [`U256::ZERO`][U256].
    ///
    /// [msg_value]: stylus_sdk::msg::value
    /// [IBeacon]: super::IBeacon
    /// [U256]: alloy_primitives::U256
    pub fn constructor(
        &mut self,
        beacon: Address,
        data: Bytes,
    ) -> Result<(), Error> {
        Erc1967Utils::upgrade_beacon_to_and_call(self, beacon, data)?;
        self.beacon.set(beacon);
        Ok(())
    }

    /// Returns the beacon.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn get_beacon(&self) -> Address {
        self.beacon.get()
    }
}

impl IProxy for BeaconProxy {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        Ok(IBeaconInterface::new(self.get_beacon()).implementation(self)?)
    }
}
