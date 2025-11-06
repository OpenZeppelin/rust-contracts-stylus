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

use alloc::{vec, vec::Vec};

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

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of a [`Trace`] contract.
#[storage]
pub struct Trace<S: Size> {
    /// Stores checkpoints in a dynamic array sorted by key.
    pub(crate) checkpoints: StorageVec<Checkpoint<S>>,
}

/// State of a [`Checkpoint`] contract.
#[storage]
pub struct Checkpoint<S: Size> {
    /// The key of the checkpoint. Used as a sorting key.
    pub(crate) key: S::KeyStorage,
    /// The value corresponding to the key.
    pub(crate) value: S::ValueStorage,
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
            self._index(pos).value.get()
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
        if pos.is_zero() {
            S::Value::ZERO
        } else {
            self._index(pos - U256::ONE).value.get()
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
            if key < self._index(mid).key.get() {
                high = mid;
            } else {
                low = mid + U256::ONE;
            }
        }

        let pos = self._upper_binary_lookup(key, low, high);

        if pos.is_zero() {
            S::Value::ZERO
        } else {
            self._index(pos - U256::ONE).value.get()
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
        if pos.is_zero() {
            S::Value::ZERO
        } else {
            self._index(pos - U256::ONE).value.get()
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
        if pos.is_zero() {
            None
        } else {
            let checkpoint = self._index(pos - U256::ONE);
            Some((checkpoint.key.get(), checkpoint.value.get()))
        }
    }

    /// Returns the number of checkpoints.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the checkpoint's state.
    pub fn length(&self) -> U256 {
        U256::from(self.checkpoints.len())
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
        let guard = self.checkpoints.get(pos).unwrap_or_else(|| {
            panic!("should get checkpoint at index `{pos}`")
        });
        (guard.key.get(), guard.value.get())
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
            let last = self._index(pos - U256::ONE);
            let last_key = last.key.get();
            let last_value = last.value.get();

            // Checkpoint keys must be non-decreasing.
            if last_key > key {
                return Err(CheckpointUnorderedInsertion {}.into());
            }

            // Update or push new checkpoint
            if last_key == key {
                self._index_mut(pos - U256::ONE).value.set(value);
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
            if self._index(mid).key.get() > key {
                high = mid;
            } else {
                low = mid + U256::ONE;
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
            if self._index(mid).key.get() < key {
                low = mid + U256::ONE;
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
    fn _index(&self, pos: U256) -> StorageGuard<'_, Checkpoint<S>> {
        self.checkpoints
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
    fn _index_mut(&mut self, pos: U256) -> StorageGuardMut<'_, Checkpoint<S>> {
        self.checkpoints
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
        let mut new_checkpoint = self.checkpoints.grow();
        new_checkpoint.key.set(key);
        new_checkpoint.value.set(value);
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{
        aliases::{U160, U96},
        Address,
    };
    use stylus_sdk::prelude::*;

    use super::*;

    unsafe impl TopLevelStorage for Trace<S160> {}

    #[public]
    impl Trace<S160> {}

    use motsu::prelude::{Contract, ResultExt};

    #[motsu::test]
    fn push(checkpoint: Contract<Trace<S160>>, alice: Address) {
        let first_key = U96::ONE;
        let first_value = uint!(11_U160);

        let second_key = uint!(2_U96);
        let second_value = uint!(22_U160);

        let third_key = uint!(3_U96);
        let third_value = uint!(33_U160);

        checkpoint
            .sender(alice)
            .push(first_key, first_value)
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(second_key, second_value)
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(third_key, third_value)
            .motsu_expect("push third");

        assert_eq!(checkpoint.sender(alice).length(), uint!(3_U256));

        assert_eq!(
            checkpoint.sender(alice).at(U32::ZERO),
            (first_key, first_value)
        );
        assert_eq!(
            checkpoint.sender(alice).at(U32::ONE),
            (second_key, second_value)
        );
        assert_eq!(
            checkpoint.sender(alice).at(uint!(2_U32)),
            (third_key, third_value)
        );
    }

    #[motsu::test]
    #[should_panic = "should get checkpoint at index `1`"]
    fn at_panics_on_exceeding_length(
        checkpoint: Contract<Trace<S160>>,
        alice: Address,
    ) {
        checkpoint.sender(alice).at(U32::ONE);
    }

    #[motsu::test]
    fn push_same_value(checkpoint: Contract<Trace<S160>>, alice: Address) {
        let first_key = U96::ONE;
        let first_value = uint!(11_U160);

        let second_key = uint!(2_U96);
        let second_value = uint!(22_U160);

        let third_key = uint!(2_U96);
        let third_value = uint!(222_U160);

        checkpoint
            .sender(alice)
            .push(first_key, first_value)
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(second_key, second_value)
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(third_key, third_value)
            .motsu_expect("push third");

        assert_eq!(
            checkpoint.sender(alice).length(),
            uint!(2_U256),
            "two checkpoints should be stored since third_value overrides second_value"
        );

        assert_eq!(
            checkpoint.sender(alice).at(U32::ZERO),
            (first_key, first_value)
        );
        assert_eq!(
            checkpoint.sender(alice).at(U32::ONE),
            (third_key, third_value)
        );
    }
    #[motsu::test]
    fn lower_lookup(checkpoint: Contract<Trace<S160>>, alice: Address) {
        checkpoint
            .sender(alice)
            .push(U96::ONE, uint!(11_U160))
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(uint!(3_U96), uint!(33_U160))
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(uint!(5_U96), uint!(55_U160))
            .motsu_expect("push third");

        assert_eq!(
            checkpoint.sender(alice).lower_lookup(uint!(2_U96)),
            uint!(33_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).lower_lookup(uint!(3_U96)),
            uint!(33_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).lower_lookup(uint!(4_U96)),
            uint!(55_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).lower_lookup(uint!(6_U96)),
            U160::ZERO
        );
    }

    #[motsu::test]
    fn upper_lookup(checkpoint: Contract<Trace<S160>>, alice: Address) {
        checkpoint
            .sender(alice)
            .push(U96::ONE, uint!(11_U160))
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(uint!(3_U96), uint!(33_U160))
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(uint!(5_U96), uint!(55_U160))
            .motsu_expect("push third");

        assert_eq!(
            checkpoint.sender(alice).upper_lookup(uint!(2_U96)),
            uint!(11_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup(U96::ONE),
            uint!(11_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup(uint!(4_U96)),
            uint!(33_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup(U96::ZERO),
            U160::ZERO
        );
    }

    #[motsu::test]
    fn upper_lookup_recent(checkpoint: Contract<Trace<S160>>, alice: Address) {
        // `upper_lookup_recent` has different optimizations for "short" (<=5)
        // and "long" (>5) checkpoint arrays.
        //
        // Validate the first approach for a short checkpoint array.
        checkpoint
            .sender(alice)
            .push(U96::ONE, uint!(11_U160))
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(uint!(3_U96), uint!(33_U160))
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(uint!(5_U96), uint!(55_U160))
            .motsu_expect("push third");

        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(uint!(2_U96)),
            uint!(11_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(U96::ONE),
            uint!(11_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(uint!(4_U96)),
            uint!(33_U160)
        );

        // Validate the second approach for a long checkpoint array.
        checkpoint
            .sender(alice)
            .push(uint!(7_U96), uint!(77_U160))
            .motsu_expect("push fourth");
        checkpoint
            .sender(alice)
            .push(uint!(9_U96), uint!(99_U160))
            .motsu_expect("push fifth");
        checkpoint
            .sender(alice)
            .push(uint!(11_U96), uint!(111_U160))
            .motsu_expect("push sixth");

        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(uint!(7_U96)),
            uint!(77_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(uint!(9_U96)),
            uint!(99_U160)
        );
        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(uint!(11_U96)),
            uint!(111_U160)
        );

        assert_eq!(
            checkpoint.sender(alice).upper_lookup_recent(U96::ZERO),
            U160::ZERO
        );
    }

    #[motsu::test]
    fn latest(checkpoint: Contract<Trace<S160>>, alice: Address) {
        assert_eq!(checkpoint.sender(alice).latest(), U160::ZERO);
        checkpoint
            .sender(alice)
            .push(U96::ONE, uint!(11_U160))
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(uint!(3_U96), uint!(33_U160))
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(uint!(5_U96), uint!(55_U160))
            .motsu_expect("push third");
        assert_eq!(checkpoint.sender(alice).latest(), uint!(55_U160));
    }

    #[motsu::test]
    fn latest_checkpoint(checkpoint: Contract<Trace<S160>>, alice: Address) {
        assert_eq!(checkpoint.sender(alice).latest_checkpoint(), None);
        checkpoint
            .sender(alice)
            .push(U96::ONE, uint!(11_U160))
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(uint!(3_U96), uint!(33_U160))
            .motsu_expect("push second");
        checkpoint
            .sender(alice)
            .push(uint!(5_U96), uint!(55_U160))
            .motsu_expect("push third");
        assert_eq!(
            checkpoint.sender(alice).latest_checkpoint(),
            Some((uint!(5_U96), uint!(55_U160)))
        );
    }

    #[motsu::test]
    fn error_when_unordered_insertion(
        checkpoint: Contract<Trace<S160>>,
        alice: Address,
    ) {
        checkpoint
            .sender(alice)
            .push(U96::ONE, uint!(11_U160))
            .motsu_expect("push first");
        checkpoint
            .sender(alice)
            .push(uint!(3_U96), uint!(33_U160))
            .motsu_expect("push second");
        let err = checkpoint
            .sender(alice)
            .push(uint!(2_U96), uint!(22_U160))
            .motsu_expect_err("should not push value lower then last one");
        assert!(matches!(
            err,
            Error::CheckpointUnorderedInsertion(
                CheckpointUnorderedInsertion {}
            )
        ));
    }
}
