//! Contract module for managing `U256` to boolean mapping in a compact and
//! efficient way, provided the keys are sequential. Largely inspired by
//! Uniswap's [merkle-distributor].
//!
//! `BitMap` packs 256 booleans across each bit of a single 256-bit slot of
//! `U256` type. Hence, booleans corresponding to 256 _sequential_ indices
//! would only consume a single slot, unlike the regular boolean which would
//! consume an entire slot for a single value.
//!
//! This results in gas savings in two ways:
//!
//! - Setting a zero value to non-zero only once every 256 times
//! - Accessing the same warm slot for every 256 _sequential_ indices
//!
//! [merkle-distributor]: https://github.com/Uniswap/merkle-distributor/blob/master/contracts/MerkleDistributor.sol
use alloy_primitives::{uint, U256};
use stylus_sdk::{
    prelude::storage,
    storage::{StorageMap, StorageU256},
};

const ONE: U256 = uint!(0x1_U256);
const HEX_FF: U256 = uint!(0xff_U256);

/// State of a [`BitMap`] contract.
#[storage]
pub struct BitMap {
    /// Inner laying mapping.
    #[allow(clippy::used_underscore_binding)]
    pub _data: StorageMap<U256, StorageU256>,
}

impl BitMap {
    /// Returns whether the bit at `index` is set.
    ///
    /// # Arguments
    ///
    /// * `index` - index of the boolean value in the bit map.
    #[must_use]
    pub fn get(&self, index: U256) -> bool {
        let bucket = Self::get_bucket(index);
        let mask = Self::get_mask(index);
        let value = self._data.get(bucket);
        (value & mask) != U256::ZERO
    }

    /// Sets the bit at `index` to the boolean `value`.
    ///
    /// # Arguments
    ///
    /// * `index` - index of boolean value in the bit map.
    /// * `value` - boolean value to set in the bit map.
    pub fn set_to(&mut self, index: U256, value: bool) {
        if value {
            self.set(index);
        } else {
            self.unset(index);
        }
    }

    /// Sets the bit at `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - index of boolean value that should be set `true`.
    pub fn set(&mut self, index: U256) {
        let bucket = Self::get_bucket(index);
        let mask = Self::get_mask(index);
        let mut value = self._data.setter(bucket);
        let prev = value.get();
        value.set(prev | mask);
    }

    /// Unsets the bit at `index`.
    ///
    /// # Arguments
    ///
    /// * `index` - index of boolean value that should be set `false`.
    pub fn unset(&mut self, index: U256) {
        let bucket = Self::get_bucket(index);
        let mask = Self::get_mask(index);
        let mut value = self._data.setter(bucket);
        let prev = value.get();
        value.set(prev & !mask);
    }

    /// Get mask of value in the bucket.
    fn get_mask(index: U256) -> U256 {
        ONE << (index & HEX_FF)
    }

    /// Get bucket index.
    fn get_bucket(index: U256) -> U256 {
        index >> 8
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{private::proptest::proptest, U256};

    use crate::utils::structs::bitmap::BitMap;

    #[motsu::test]
    fn set_value() {
        proptest!(|(value: U256)| {
            let mut bit_map = BitMap::default();
            assert!(!bit_map.get(value));
            bit_map.set(value);
            assert!(bit_map.get(value));
        });
    }

    #[motsu::test]
    fn unset_value() {
        proptest!(|(value: U256)| {
            let mut bit_map = BitMap::default();
            bit_map.set(value);
            assert!(bit_map.get(value));
            bit_map.unset(value);
            assert!(!bit_map.get(value));
        });
    }

    #[motsu::test]
    fn set_to_value() {
        proptest!(|(value: U256)| {
            let mut bit_map = BitMap::default();
            bit_map.set_to(value, true);
            assert!(bit_map.get(value));
            bit_map.set_to(value, false);
            assert!(!bit_map.get(value));
        });
    }
}
