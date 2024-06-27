//! Contract module for checkpointing values as they
//! change at different points in time.
//! Lets to look up past values by
//! block number.
//! To create a history of checkpoints,
//! define a variable type [`Trace160`] in your contract, and store a
//! new checkpoint for the current transaction block using the
//! [`Trace160::push`] function.
use alloy_primitives::{uint, Uint, U256, U32};
use alloy_sol_types::sol;
use stylus_proc::{sol_storage, SolidityError};
use stylus_sdk::storage::{StorageGuard, StorageGuardMut};

use crate::utils::math::alloy::Math;

// TODO: add generics for other pairs (uint32, uint224) and (uint48, uint208).
// Logic should be the same.
type U96 = Uint<96, 2>;
type U160 = Uint<160, 3>;

sol! {
    /// A value was attempted to be inserted on a past checkpoint.
    #[derive(Debug)]
    error CheckpointUnorderedInsertion();
}

/// An error that occurred while calling [`Trace160`] checkpoint contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// A value was attempted to be inserted on a past checkpoint.
    CheckpointUnorderedInsertion(CheckpointUnorderedInsertion),
}

sol_storage! {
    /// State of checkpoint library contract.
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Trace160 {
        /// Stores checkpoints in a dynamic array sorted by key.
        Checkpoint160[] _checkpoints;
    }

    /// State of a single checkpoint.
    pub struct Checkpoint160 {
        /// Key of checkpoint. Used as a sorting key.
        uint96 _key;
        /// Value corresponding to the key.
        uint160 _value;
    }
}

impl Trace160 {
    /// Pushes a (`key`, `value`) pair into a Trace160 so that it is
    /// stored as the checkpoint.
    ///
    /// Returns previous value and new value.
    ///
    /// IMPORTANT: Never accept `key` as a user input, since an arbitrary
    /// `U96::MAX` key set will disable the library.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `key` - Last checkpoint key to insert.
    /// * `value` - Checkpoint value corresponding to insertion `key`.
    ///
    /// # Errors
    ///
    /// If the `key` is lower than previously pushed checkpoint's key error
    /// [`Error::CheckpointUnorderedInsertion`] is returned (necessary to
    /// maintain sorted order).
    pub fn push(
        &mut self,
        key: U96,
        value: U160,
    ) -> Result<(U160, U160), Error> {
        self._insert(key, value)
    }

    /// Returns the value in the first (oldest) checkpoint with key
    /// greater or equal than the search key, or zero if there is none.
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    /// * `key` - Checkpoint's key to lookup.
    pub fn lower_lookup(&self, key: U96) -> U160 {
        let len = self.length();
        let pos = self._lower_binary_lookup(key, U256::ZERO, len);
        if pos == len {
            U160::ZERO
        } else {
            self._access(pos)._value.get()
        }
    }

    /// Returns the value in the last (most recent) checkpoint with key
    /// lower or equal than the search key, or zero if there is none.
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    /// * `key` - Checkpoint's key to lookup.
    pub fn upper_lookup(&self, key: U96) -> U160 {
        let len = self.length();
        let pos = self._upper_binary_lookup(key, U256::ZERO, len);
        if pos == U256::ZERO {
            U160::ZERO
        } else {
            self._access(pos - uint!(1_U256))._value.get()
        }
    }

    /// Returns the value in the last (most recent) checkpoint with key
    /// lower or equal than the search key, or zero if there is none.
    ///
    /// This is a variant of [`Self::upper_lookup`] that is optimized to find
    /// "recent" checkpoint (checkpoints with high keys).
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    /// * `key` - Checkpoint's key to query.
    pub fn upper_lookup_recent(&self, key: U96) -> U160 {
        let len = self.length();

        let mut low = U256::ZERO;
        let mut high = len;

        if len > uint!(5_U256) {
            let mid = len - len.sqrt();
            if key < self._access(mid)._key.get() {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }

        let pos = self._upper_binary_lookup(key, low, high);

        if pos == U256::ZERO {
            U160::ZERO
        } else {
            self._access(pos - uint!(1_U256))._value.get()
        }
    }

    /// Returns the value in the most recent checkpoint, or zero if
    /// there are no checkpoints.
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    pub fn latest(&self) -> U160 {
        let pos = self.length();
        if pos == U256::ZERO {
            U160::ZERO
        } else {
            self._access(pos - uint!(1_U256))._value.get()
        }
    }

    /// Returns whether there is a checkpoint in the structure (i.g. it
    /// is not empty), and if so, the key and value in the most recent
    /// checkpoint.
    /// Otherwise, [`None`] will be returned.
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    pub fn latest_checkpoint(&self) -> Option<(U96, U160)> {
        let pos = self.length();
        if pos == U256::ZERO {
            None
        } else {
            let checkpoint = self._access(pos - uint!(1_U256));
            Some((checkpoint._key.get(), checkpoint._value.get()))
        }
    }

    /// Returns the number of checkpoints.
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    pub fn length(&self) -> U256 {
        U256::from(self._checkpoints.len())
    }

    /// Returns checkpoint at given position.
    ///
    /// # Arguments
    ///
    /// * `&self` - read access to the checkpoint's state.
    /// * `pos` - index of the checkpoint.
    pub fn at(&self, pos: U32) -> Checkpoint160 {
        unsafe { self._checkpoints.get(pos).unwrap().into_raw() }
    }

    /// Pushes a (`key`, `value`) pair into an ordered list of
    /// checkpoints, either by inserting a new checkpoint, or by updating
    /// the last one.
    /// Returns previous value and new value.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `key` - Last checkpoint key to insert.
    /// * `value` - Checkpoint value corresponding to insertion `key`.
    ///
    /// # Errors
    ///
    /// To maintain sorted order if the `key` is lower than
    /// previously inserted error [`Error::CheckpointUnorderedInsertion`] is
    /// returned.
    fn _insert(
        &mut self,
        key: U96,
        value: U160,
    ) -> Result<(U160, U160), Error> {
        let pos = self.length();
        if pos > U256::ZERO {
            let last = self._access(pos - uint!(1_U256));
            let last_key = last._key.get();
            let last_value = last._value.get();

            // Checkpoint keys must be non-decreasing.
            if last_key > key {
                return Err(CheckpointUnorderedInsertion {}.into());
            }

            // Update or push new checkpoint
            if last_key == key {
                self._access_mut(pos - uint!(1_U256))._value.set(value);
            } else {
                self._unchecked_push(key, value);
            }
            Ok((last_value, value))
        } else {
            self._unchecked_push(key, value);
            Ok((U160::ZERO, value))
        }
    }

    /// Return the index of the last (most recent) checkpoint with key
    /// lower or equal than the search key, or `high` if there is none.
    /// Indexes `low` and `high` define a section where to do the search, with
    /// inclusive `low` and exclusive `high`.
    ///
    /// WARNING: `high` should not be greater than the array's length.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint key to lookup.
    /// * `low` - inclusive index where search begins.
    /// * `high` - exclusive index where search ends.
    fn _upper_binary_lookup(
        &self,
        key: U96,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = low.average(high);
            if self._access(mid)._key.get() > key {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }
        high
    }

    /// Return the index of the first (oldest) checkpoint with key is
    /// greater or equal than the search key, or `high` if there is none.
    /// Indexes `low` and `high` define a section where to do the search, with
    /// inclusive `low` and exclusive `high`.
    ///
    /// WARNING: `high` should not be greater than the array's length.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint key to lookup.
    /// * `low` - inclusive index where search begins.
    /// * `high` - exclusive index where search ends.
    fn _lower_binary_lookup(
        &self,
        key: U96,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = low.average(high);
            if self._access(mid)._key.get() < key {
                low = mid + uint!(1_U256);
            } else {
                high = mid;
            }
        }
        high
    }

    /// Immutable access on an element of the checkpoint's array.
    /// The position is assumed to be within bounds.
    /// Panic when out of bounds.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `pos` - index of the checkpoint.
    fn _access(&self, pos: U256) -> StorageGuard<Checkpoint160> {
        self._checkpoints
            .get(pos)
            .unwrap_or_else(|| panic!("should get checkpoint at index `{pos}`"))
    }

    /// Mutable access on an element of the checkpoint's array.
    /// The position is assumed to be within bounds.
    /// Panic when out of bounds.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `pos` - index of the checkpoint.
    fn _access_mut(&mut self, pos: U256) -> StorageGuardMut<Checkpoint160> {
        self._checkpoints
            .setter(pos)
            .unwrap_or_else(|| panic!("should get checkpoint at index `{pos}`"))
    }

    /// Append checkpoint without checking if sorted order pertains after.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `key` - Checkpoint key to insert.
    /// * `value` - Checkpoint value corresponding to insertion `key`.
    fn _unchecked_push(&mut self, key: U96, value: U160) {
        let mut new_checkpoint = self._checkpoints.grow();
        new_checkpoint._key.set(key);
        new_checkpoint._value.set(value);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::uint;

    use crate::utils::structs::checkpoints::{
        CheckpointUnorderedInsertion, Error, Trace160,
    };

    #[motsu::test]
    fn push(checkpoint: Trace160) {
        let first_key = uint!(1_U96);
        let first_value = uint!(11_U160);

        let second_key = uint!(2_U96);
        let second_value = uint!(22_U160);

        let third_key = uint!(3_U96);
        let third_value = uint!(33_U160);

        checkpoint.push(first_key, first_value).expect("push first");
        checkpoint.push(second_key, second_value).expect("push second");
        checkpoint.push(third_key, third_value).expect("push third");

        assert_eq!(checkpoint.length(), uint!(3_U256));

        assert_eq!(checkpoint.at(uint!(0_U32))._key.get(), first_key);
        assert_eq!(checkpoint.at(uint!(0_U32))._value.get(), first_value);

        assert_eq!(checkpoint.at(uint!(1_U32))._key.get(), second_key);
        assert_eq!(checkpoint.at(uint!(1_U32))._value.get(), second_value);

        assert_eq!(checkpoint.at(uint!(2_U32))._key.get(), third_key);
        assert_eq!(checkpoint.at(uint!(2_U32))._value.get(), third_value);
    }

    #[motsu::test]
    fn lower_lookup(checkpoint: Trace160) {
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");

        assert_eq!(checkpoint.lower_lookup(uint!(2_U96)), uint!(33_U160));
        assert_eq!(checkpoint.lower_lookup(uint!(3_U96)), uint!(33_U160));
        assert_eq!(checkpoint.lower_lookup(uint!(4_U96)), uint!(55_U160));
        assert_eq!(checkpoint.lower_lookup(uint!(6_U96)), uint!(0_U160));
    }

    #[motsu::test]
    fn upper_lookup(checkpoint: Trace160) {
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");

        assert_eq!(checkpoint.upper_lookup(uint!(2_U96)), uint!(11_U160));
        assert_eq!(checkpoint.upper_lookup(uint!(1_U96)), uint!(11_U160));
        assert_eq!(checkpoint.upper_lookup(uint!(4_U96)), uint!(33_U160));
        assert_eq!(checkpoint.upper_lookup(uint!(0_U96)), uint!(0_U160));
    }

    #[motsu::test]
    fn upper_lookup_recent(checkpoint: Trace160) {
        // Since `upper_lookup_recent` optimized for higher keys (>5) compare to
        // `upper_lookup`. All test key values will be higher then 5.
        checkpoint.push(uint!(11_U96), uint!(111_U160)).expect("push first");
        checkpoint.push(uint!(33_U96), uint!(333_U160)).expect("push second");
        checkpoint.push(uint!(55_U96), uint!(555_U160)).expect("push third");

        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(22_U96)),
            uint!(111_U160)
        );
        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(11_U96)),
            uint!(111_U160)
        );
        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(44_U96)),
            uint!(333_U160)
        );
        assert_eq!(checkpoint.upper_lookup_recent(uint!(0_U96)), uint!(0_U160));
    }

    #[motsu::test]
    fn latest(checkpoint: Trace160) {
        assert_eq!(checkpoint.latest(), uint!(0_U160));
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");
        assert_eq!(checkpoint.latest(), uint!(55_U160));
    }

    #[motsu::test]
    fn latest_checkpoint(checkpoint: Trace160) {
        assert_eq!(checkpoint.latest_checkpoint(), None);
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");
        assert_eq!(
            checkpoint.latest_checkpoint(),
            Some((uint!(5_U96), uint!(55_U160)))
        );
    }

    #[motsu::test]
    fn error_when_unordered_insertion(checkpoint: Trace160) {
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        let err = checkpoint
            .push(uint!(2_U96), uint!(22_U160))
            .expect_err("should not push value lower then last one");
        assert!(matches!(
            err,
            Error::CheckpointUnorderedInsertion(
                CheckpointUnorderedInsertion {}
            )
        ));
    }
}
