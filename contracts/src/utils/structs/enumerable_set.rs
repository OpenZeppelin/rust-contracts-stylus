//! TODO: docs
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
use stylus_sdk::{
    prelude::*,
    storage::{
        StorageAddress, StorageMap, StorageType, StorageU256, StorageVec,
    },
};

/// TODO: docs
#[storage]
pub struct EnumerableSet {
    /// TODO: docs
    values: StorageVec<StorageAddress>,
    /// TODO: docs
    positions: StorageMap<Address, StorageU256>,
}

impl EnumerableSet {
    /// Adds a value to a set. O(1).
    ///
    /// Returns true if the `value` was added to the set, that is if it was not
    /// already present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to add to the set.
    pub fn add(&mut self, _value: Address) {
        unimplemented!()
    }

    /// Removes a `value` from a set. O(1).
    ///
    /// Returns true if the `value` was removed from the set, that is if it was
    /// present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to remove from the set.
    pub fn remove(&mut self, _value: Address) {}

    /// Returns true if the `value` is in the set. O(1).
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `value` - The value to check for in the set.
    pub fn contains(&self, value: Address) -> bool {
        let position = self.positions.getter(value).get();
        // When the position is [`U256::ZERO`], the value is either on a first
        // index or not in the set. When the length is [`U256::ZERO`], the set
        // is empty, which means the value is not in the set.
        // <https://docs.rs/stylus-sdk/0.9.0/stylus_sdk/storage/struct.StorageMap.html#method.getter>
        !position.is_zero() || !self.length().is_zero()
    }

    /// Returns the number of values in the set. O(1).
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    pub fn length(&self) -> U256 {
        U256::from(self.values.len())
    }

    /// Returns the value stored at position `index` in the set. O(1).
    ///
    /// Note that there are no guarantees on the ordering of values inside the
    /// array, and it may change when more values are added or removed.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `index` - The index of the value to return.
    pub fn at(&self, _index: U256) -> Option<Address> {
        self.values.get(_index)
    }

    /// Returns the entire set in an array.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    pub fn values(&self) -> Vec<Address> {
        unimplemented!()
    }
}
