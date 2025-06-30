use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, B256};
use stylus_sdk::prelude::*;

#[storage]
pub struct StorageSlot {}

#[public]
impl StorageSlot {
    /// Returns an [`Address`] with member `value` located at `slot`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `slot` - The slot to get the address from.
    pub fn get_address_slot(&self, _slot: B256) -> Address {
        unimplemented!()
    }

    /// TODO: docs
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `slot` - The slot to set the address to.
    /// * `value` - The address to set.
    pub fn set_address_slot(&self, _slot: B256, _value: Address) {
        unimplemented!()
    }
}
