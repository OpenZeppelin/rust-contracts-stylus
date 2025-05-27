//! TODO: docs
use alloc::{vec, vec::Vec};

use alloy_primitives::U256;
use stylus_sdk::{
    prelude::*,
    storage::{StorageKey, StorageMap, StorageType, StorageU256, StorageVec},
};

/// TODO: docs
#[storage]
pub struct EnumerableSet<K: StorageKey, V: StorageType> {
    /// TODO: docs
    values: StorageVec<V>,
    /// TODO: docs
    _positions: StorageMap<K, StorageU256>,
}

impl<K: StorageKey, V: StorageType> EnumerableSet<K, V> {
    /// TODO: docs
    pub fn add(&mut self, _value: K) {
        unimplemented!()
    }

    /// TODO: docs
    pub fn remove(&mut self, _value: K) {}

    /// TODO: docs
    pub fn contains(&self, _value: K) -> bool {
        unimplemented!()
    }

    /// TODO: docs
    pub fn at(&self, _index: U256) -> Option<K> {
        unimplemented!()
    }

    /// TODO: docs
    pub fn length(&self) -> U256 {
        U256::from(self.values.len())
    }

    /// TODO: docs
    pub fn values(&self) -> Vec<K> {
        // self._values.get()
        unimplemented!()
    }
}
