use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
use stylus_sdk::{abi::Bytes, prelude::*, storage::StorageAddress};

use crate::proxy::{
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
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `beacon` - The beacon address.
    /// * `data` - The data to pass to the beacon.
    pub fn constructor(
        &mut self,
        beacon: Address,
        data: Bytes,
    ) -> Result<(), Error> {
        self.beacon.set(beacon);
        Erc1967Utils::upgrade_beacon_to_and_call(self, beacon, data)
    }
}
