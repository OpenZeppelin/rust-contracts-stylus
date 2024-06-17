//! Contract module for managing `U256` to boolean mapping in a compact and
//! efficient way, provided the keys are sequential. Largely inspired by Uniswap's <https://github.com/Uniswap/merkle-distributor/blob/master/contracts/MerkleDistributor.sol>[merkle-distributor].
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
use alloy_primitives::{Uint, U256};
use stylus_proc::sol_storage;

sol_storage! {
    /// State of bit map.
    pub struct BitMap {
        /// Inner laying mapping.
        mapping(uint256 => uint256) _data;
    }
}

impl BitMap {
    /// Returns whether the bit at `index` is set.
    ///
    /// # Arguments
    ///
    /// * `index` - index of boolean value at the bit map.
    #[must_use]
    pub fn get(&self, index: U256) -> bool {
        let bucket = index >> 8;
        let mask = U256::from(1) << (index & U256::from(0xff));
        let value = self._data.get(bucket);
        (value & mask) != U256::ZERO
    }

    /// Sets the bit at `index` to the boolean `value`.
    ///
    /// # Arguments
    ///
    /// * `index` - index of boolean value at the bit map.
    /// * `value` - boolean value to set into the bit map.
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
        U256::from(1) << (index & U256::from(0xff))
    }

    /// Get bucket index.
    fn get_bucket(index: U256) -> U256 {
        index >> 8
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{
        private::proptest::{
            prelude::{Arbitrary, ProptestConfig},
            proptest,
        },
        U256,
    };
    use stylus_sdk::{prelude::*, storage::StorageMap};

    use crate::utils::structs::bitmap::BitMap;

    impl Default for BitMap {
        fn default() -> Self {
            let root = U256::ZERO;
            BitMap { _data: unsafe { StorageMap::new(root, 0) } }
        }
    }

    #[motsu::test]
    fn set_value() {
        proptest!(|(value: U256)| {
            let mut bit_map = BitMap::default();
            assert_eq!(bit_map.get(value), false);
            bit_map.set(value);
            assert_eq!(bit_map.get(value), true);
        });
    }

    #[motsu::test]
    fn unset_value() {
        proptest!(|(value: U256)| {
            let mut bit_map = BitMap::default();
            bit_map.set(value);
            assert_eq!(bit_map.get(value), true);
            bit_map.unset(value);
            assert_eq!(bit_map.get(value), false);
        });
    }

    #[motsu::test]
    fn set_to_value() {
        proptest!(|(value: U256)| {
            let mut bit_map = BitMap::default();
            bit_map.set_to(value, true);
            assert_eq!(bit_map.get(value), true);
            bit_map.set_to(value, false);
            assert_eq!(bit_map.get(value), false);
        });
    }
}
