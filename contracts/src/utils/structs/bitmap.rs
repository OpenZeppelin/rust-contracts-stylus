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
use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, U256};
use stylus_sdk::{
    prelude::*,
    storage::{StorageMap, StorageU256},
};
const ONE: U256 = uint!(0x1_U256);
const HEX_FF: U256 = uint!(0xff_U256);

/// State of a [`BitMap`] contract.
#[storage]
pub struct BitMap {
    /// Inner laying mapping.
    pub(crate) data: StorageMap<U256, StorageU256>,
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
        let value = self.data.get(bucket);
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
        let mut value = self.data.setter(bucket);
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
        let mut value = self.data.setter(bucket);
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{
        private::proptest::{prop_assert, proptest},
        Address, U256,
    };
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::{public, TopLevelStorage};

    use crate::utils::structs::bitmap::BitMap;

    unsafe impl TopLevelStorage for BitMap {}

    #[public]
    impl BitMap {}

    #[motsu::test]
    fn set_value() {
        proptest!(|(value: U256, alice: Address)| {
            let bit_map = Contract::<BitMap>::new();
            let mut bit_map = bit_map.sender(alice);
            prop_assert!(!bit_map.get(value));
            bit_map.set(value);
            prop_assert!(bit_map.get(value));
        });
    }

    #[motsu::test]
    fn unset_value() {
        proptest!(|(value: U256, alice: Address)| {
            let bit_map = Contract::<BitMap>::new();
            let mut bit_map = bit_map.sender(alice);
            bit_map.set(value);
            prop_assert!(bit_map.get(value));
            bit_map.unset(value);
            prop_assert!(!bit_map.get(value));
        });
    }

    #[motsu::test]
    fn set_to_value() {
        proptest!(|(value: U256, alice: Address)| {
            let bit_map = Contract::<BitMap>::new();
            let mut bit_map = bit_map.sender(alice);
            bit_map.set_to(value, true);
            prop_assert!(bit_map.get(value));
            bit_map.set_to(value, false);
            prop_assert!(!bit_map.get(value));
        });
    }
}
