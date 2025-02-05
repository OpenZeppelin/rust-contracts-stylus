//! Contract for checkpointing values as they change at different points in
//! time, and later looking up and later looking up past values by block number.
//!
//! To create a history of checkpoints, define a variable type [`Trace`]
//! in your contract.
//! Types [`S160`], [`S160`] and [`S160`] can be used to
//! define sizes for key and value.
//! Then store a new checkpoint for the current
//! transaction block using the [`Trace::push`] function.
pub mod generic_size;

use alloy_primitives::{uint, U256, U32};
pub use generic_size::{Size, S160, S208, S224};
pub use sol::*;
use stylus_sdk::{
    call::MethodError,
    prelude::*,
    storage::{StorageGuard, StorageGuardMut, StorageVec},
};

use crate::utils::{
    math::alloy::Math,
    structs::checkpoints::generic_size::{Accessor, Num},
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// A value was attempted to be inserted into a past checkpoint.
        #[derive(Debug)]
        error CheckpointUnorderedInsertion();
    }
}

/// An error that occurred while calling the [`Trace`] checkpoint contract.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// A value was attempted to be inserted into a past checkpoint.
    CheckpointUnorderedInsertion(CheckpointUnorderedInsertion),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of a [`Trace`] contract.
#[storage]
pub struct Trace<S: Size> {
    /// Stores checkpoints in a dynamic array sorted by key.
    #[allow(clippy::used_underscore_binding)]
    pub _checkpoints: StorageVec<Checkpoint<S>>,
}

/// State of a [`Checkpoint`] contract.
#[storage]
pub struct Checkpoint<S: Size> {
    /// The key of the checkpoint. Used as a sorting key.
    #[allow(clippy::used_underscore_binding)]
    pub _key: S::KeyStorage,
    /// The value corresponding to the key.
    #[allow(clippy::used_underscore_binding)]
    pub _value: S::ValueStorage,
}

impl<S: Size> Trace<S> {
    /// Pushes a (`key`, `value`) pair into a `Trace` so that it is
    /// stored as the checkpoint.
    ///
    /// Returns the previous value and the new value as an ordered pair.
    ///
    /// IMPORTANT: Never accept `key` as user input, since an arbitrary
    /// `U96::MAX` key set will disable the library.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `key` - Latest checkpoint key to insert.
    /// * `value` - Checkpoint value corresponding to `key`.
    ///
    /// # Errors
    ///
    /// * [`Error::CheckpointUnorderedInsertion`] - If the `key` is lower than
    ///   previously pushed checkpoint's key (necessary to maintain sorted
    ///   order).
    pub fn push(
        &mut self,
        key: S::Key,
        value: S::Value,
    ) -> Result<(S::Value, S::Value), Error> {
        self._insert(key, value)
    }

    /// Returns the value in the first (oldest) checkpoint with key greater or
    /// equal than the search key, or `S::Value::ZERO` if there is none.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint's key to lookup.
    pub fn lower_lookup(&self, key: S::Key) -> S::Value {
        let len = self.length();
        let pos = self._lower_binary_lookup(key, U256::ZERO, len);
        if pos == len {
            S::Value::ZERO
        } else {
            self._index(pos)._value.get()
        }
    }

    /// Returns the value in the last (most recent) checkpoint with key
    /// lower or equal than the search key, or `S::Value::ZERO` if there is
    /// none.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint's key to lookup.
    pub fn upper_lookup(&self, key: S::Key) -> S::Value {
        let len = self.length();
        let pos = self._upper_binary_lookup(key, U256::ZERO, len);
        if pos == U256::ZERO {
            S::Value::ZERO
        } else {
            self._index(pos - uint!(1_U256))._value.get()
        }
    }

    /// Returns the value in the last (most recent) checkpoint with key lower or
    /// equal than the search key, or `S::Value::ZERO` if there is none.
    ///
    /// This is a variant of [`Self::upper_lookup`] that is optimized to find
    /// "recent" checkpoints (checkpoints with high keys).
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint's key to query.
    pub fn upper_lookup_recent(&self, key: S::Key) -> S::Value {
        let len = self.length();

        let mut low = U256::ZERO;
        let mut high = len;

        if len > uint!(5_U256) {
            let mid = len - len.sqrt();
            if key < self._index(mid)._key.get() {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }

        let pos = self._upper_binary_lookup(key, low, high);

        if pos == U256::ZERO {
            S::Value::ZERO
        } else {
            self._index(pos - uint!(1_U256))._value.get()
        }
    }

    /// Returns the value in the most recent checkpoint, or `S::Value::ZERO` if
    /// there are no checkpoints.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    pub fn latest(&self) -> S::Value {
        let pos = self.length();
        if pos == U256::ZERO {
            S::Value::ZERO
        } else {
            self._index(pos - uint!(1_U256))._value.get()
        }
    }

    /// Returns whether there is a checkpoint in the structure (i.g. it is not
    /// empty), and if so, the key and value in the most recent checkpoint.
    /// Otherwise, [`None`] will be returned.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    pub fn latest_checkpoint(&self) -> Option<(S::Key, S::Value)> {
        let pos = self.length();
        if pos == U256::ZERO {
            None
        } else {
            let checkpoint = self._index(pos - uint!(1_U256));
            Some((checkpoint._key.get(), checkpoint._value.get()))
        }
    }

    /// Returns the number of checkpoints.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    pub fn length(&self) -> U256 {
        U256::from(self._checkpoints.len())
    }

    /// Returns checkpoint at given position.
    ///
    /// # Panics
    ///
    /// If `pos` exceeds [`Self::length`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `pos` - Index of the checkpoint.
    pub fn at(&self, pos: U32) -> (S::Key, S::Value) {
        let guard = self._checkpoints.get(pos).unwrap_or_else(|| {
            panic!("should get checkpoint at index `{pos}`")
        });
        (guard._key.get(), guard._value.get())
    }

    /// Pushes a (`key`, `value`) pair into an ordered list of checkpoints,
    /// either by inserting a new checkpoint, or by updating the last one.
    /// Returns the previous value and the new value as an ordered pair.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `key` - The key of the checkpoint to insert.
    /// * `value` - Checkpoint value corresponding to insertion `key`.
    ///
    /// # Errors
    ///
    /// * [`Error::CheckpointUnorderedInsertion`] - If the `key` is lower than
    ///   the previously inserted one.
    fn _insert(
        &mut self,
        key: S::Key,
        value: S::Value,
    ) -> Result<(S::Value, S::Value), Error> {
        let pos = self.length();
        if pos > U256::ZERO {
            let last = self._index(pos - uint!(1_U256));
            let last_key = last._key.get();
            let last_value = last._value.get();

            // Checkpoint keys must be non-decreasing.
            if last_key > key {
                return Err(CheckpointUnorderedInsertion {}.into());
            }

            // Update or push new checkpoint
            if last_key == key {
                self._index_mut(pos - uint!(1_U256))._value.set(value);
            } else {
                self._unchecked_push(key, value);
            }
            Ok((last_value, value))
        } else {
            self._unchecked_push(key, value);
            Ok((S::Value::ZERO, value))
        }
    }

    /// Return the index of the last (most recent) checkpoint with key lower or
    /// equal than the search key, or `high` if there is none.
    ///
    /// Indexes `low` and `high` define a section where to do the search, with
    /// inclusive `low` and exclusive `high`.
    ///
    /// WARNING: `high` should not be greater than the array's length.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint key to lookup.
    /// * `low` - Inclusive index where search begins.
    /// * `high` - Exclusive index where search ends.
    fn _upper_binary_lookup(
        &self,
        key: S::Key,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = low.average(high);
            if self._index(mid)._key.get() > key {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }
        high
    }

    /// Return the index of the first (oldest) checkpoint with key is greater or
    /// equal than the search key, or `high` if there is none.
    ///
    /// Indexes `low` and `high` define a section where to do the search, with
    /// inclusive `low` and exclusive `high`.
    ///
    /// WARNING: `high` should not be greater than the array's length.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `key` - Checkpoint key to lookup.
    /// * `low` - Inclusive index where search begins.
    /// * `high` - Exclusive index where search ends.
    fn _lower_binary_lookup(
        &self,
        key: S::Key,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = low.average(high);
            if self._index(mid)._key.get() < key {
                low = mid + uint!(1_U256);
            } else {
                high = mid;
            }
        }
        high
    }

    /// Immutable access on an element of the checkpoint's array. The position
    /// is assumed to be within bounds.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    /// * `pos` - Index of the checkpoint.
    ///
    /// # Panics
    ///
    /// * If `pos` exceeds [`Self::length`].
    fn _index(&self, pos: U256) -> StorageGuard<Checkpoint<S>> {
        self._checkpoints
            .get(pos)
            .unwrap_or_else(|| panic!("should get checkpoint at index `{pos}`"))
    }

    /// Mutable access on an element of the checkpoint's array. The position is
    /// assumed to be within bounds.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `pos` - Index of the checkpoint.
    ///
    /// # Panics
    ///
    /// * If `pos` exceeds [`Self::length`].
    fn _index_mut(&mut self, pos: U256) -> StorageGuardMut<Checkpoint<S>> {
        self._checkpoints
            .setter(pos)
            .unwrap_or_else(|| panic!("should get checkpoint at index `{pos}`"))
    }

    /// Append a checkpoint without checking if the sorted order is kept.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the checkpoint's state.
    /// * `key` - Checkpoint key to insert.
    /// * `value` - Checkpoint value corresponding to insertion `key`.
    fn _unchecked_push(&mut self, key: S::Key, value: S::Value) {
        let mut new_checkpoint = self._checkpoints.grow();
        new_checkpoint._key.set(key);
        new_checkpoint._value.set(value);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::uint;

    use crate::utils::structs::checkpoints::{
        generic_size::S160, CheckpointUnorderedInsertion, Error, Trace,
    };

    #[motsu::test]
    fn push(checkpoint: Trace<S160>) {
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

        assert_eq!(checkpoint.at(uint!(0_U32)), (first_key, first_value));
        assert_eq!(checkpoint.at(uint!(1_U32)), (second_key, second_value));
        assert_eq!(checkpoint.at(uint!(2_U32)), (third_key, third_value));
    }

    #[motsu::test]
    fn push_same_value(checkpoint: Trace<S160>) {
        let first_key = uint!(1_U96);
        let first_value = uint!(11_U160);

        let second_key = uint!(2_U96);
        let second_value = uint!(22_U160);

        let third_key = uint!(2_U96);
        let third_value = uint!(222_U160);

        checkpoint.push(first_key, first_value).expect("push first");
        checkpoint.push(second_key, second_value).expect("push second");
        checkpoint.push(third_key, third_value).expect("push third");

        assert_eq!(
            checkpoint.length(),
            uint!(2_U256),
            "two checkpoints should be stored since third_value overrides second_value"
        );

        assert_eq!(checkpoint.at(uint!(0_U32)), (first_key, first_value));
        assert_eq!(checkpoint.at(uint!(1_U32)), (third_key, third_value));
    }

    #[motsu::test]
    fn lower_lookup(checkpoint: Trace<S160>) {
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");

        assert_eq!(checkpoint.lower_lookup(uint!(2_U96)), uint!(33_U160));
        assert_eq!(checkpoint.lower_lookup(uint!(3_U96)), uint!(33_U160));
        assert_eq!(checkpoint.lower_lookup(uint!(4_U96)), uint!(55_U160));
        assert_eq!(checkpoint.lower_lookup(uint!(6_U96)), uint!(0_U160));
    }

    #[motsu::test]
    fn upper_lookup(checkpoint: Trace<S160>) {
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");

        assert_eq!(checkpoint.upper_lookup(uint!(2_U96)), uint!(11_U160));
        assert_eq!(checkpoint.upper_lookup(uint!(1_U96)), uint!(11_U160));
        assert_eq!(checkpoint.upper_lookup(uint!(4_U96)), uint!(33_U160));
        assert_eq!(checkpoint.upper_lookup(uint!(0_U96)), uint!(0_U160));
    }

    #[motsu::test]
    fn upper_lookup_recent(checkpoint: Trace<S160>) {
        // `upper_lookup_recent` has different optimizations for "short" (<=5)
        // and "long" (>5) checkpoint arrays.
        //
        // Validate the first approach for a short checkpoint array.
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");

        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(2_U96)),
            uint!(11_U160)
        );
        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(1_U96)),
            uint!(11_U160)
        );
        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(4_U96)),
            uint!(33_U160)
        );

        // Validate the second approach for a long checkpoint array.
        checkpoint.push(uint!(7_U96), uint!(77_U160)).expect("push fourth");
        checkpoint.push(uint!(9_U96), uint!(99_U160)).expect("push fifth");
        checkpoint.push(uint!(11_U96), uint!(111_U160)).expect("push sixth");

        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(7_U96)),
            uint!(77_U160)
        );
        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(9_U96)),
            uint!(99_U160)
        );
        assert_eq!(
            checkpoint.upper_lookup_recent(uint!(11_U96)),
            uint!(111_U160)
        );

        assert_eq!(checkpoint.upper_lookup_recent(uint!(0_U96)), uint!(0_U160));
    }

    #[motsu::test]
    fn latest(checkpoint: Trace<S160>) {
        assert_eq!(checkpoint.latest(), uint!(0_U160));
        checkpoint.push(uint!(1_U96), uint!(11_U160)).expect("push first");
        checkpoint.push(uint!(3_U96), uint!(33_U160)).expect("push second");
        checkpoint.push(uint!(5_U96), uint!(55_U160)).expect("push third");
        assert_eq!(checkpoint.latest(), uint!(55_U160));
    }

    #[motsu::test]
    fn latest_checkpoint(checkpoint: Trace<S160>) {
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
    fn error_when_unordered_insertion(checkpoint: Trace<S160>) {
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
