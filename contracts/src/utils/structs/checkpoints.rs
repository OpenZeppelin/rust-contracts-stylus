//! Contract module for checkpointing values as they
//! change at different points in time, and later looking up past values by
//! block number. See {Votes} as an example. To create a history of checkpoints
//! define a variable type `Checkpoints.Trace*` in your contract, and store a
//! new checkpoint for the current transaction block using the {push} function.
use alloy_primitives::{uint, Uint, U256, U32};
use alloy_sol_types::{sol};
use stylus_proc::{sol_storage, solidity_storage, SolidityError};
use stylus_sdk::prelude::StorageType;

use crate::utils::math::alloy::Math;

// type U96 = Uint<96, 2>;
// type U160= Uint<160, 3>;

sol! {
    /// A value was attempted to be inserted on a past checkpoint.
    #[derive(Debug)]
    error CheckpointUnorderedInsertion();
}

#[derive(SolidityError, Debug)]
pub enum Error {
    CheckpointUnorderedInsertion(CheckpointUnorderedInsertion),
}

#[solidity_storage]
pub struct Trace<T: Size> {
    _checkpoints: stylus_sdk::storage::StorageVec<Checkpoint<T>>,
}

#[solidity_storage]
pub struct Checkpoint<T: Size> {
    _key: T::KeyStorage,
    _value: T::ValueStorage,
}

pub trait Size {
    type KeyStorage: for<'a> StorageType<Wraps<'a> = Self::Key>
        + Accessor<Wrap=Self::Key>;
    type ValueStorage: for<'a> StorageType<Wraps<'a> = Self::Value>
        + Accessor<Wrap=Self::Value>;
    type Key: Num;
    type Value: Num;
}

pub type Size160 = SpecificSize<96, 2, 160, 3>;

struct SpecificSize<
    const KEY_BITS: usize,
    const KEY_LIMBS: usize,
    const VALUE_BITS: usize,
    const VALUE_LIMBS: usize,
>;
impl<const KB: usize, const KL: usize, const VB: usize, const VL: usize> Size
    for SpecificSize<KB, KL, VB, VL>
{
    type KeyStorage = stylus_sdk::storage::StorageUint<KB, KL>;
    type ValueStorage = stylus_sdk::storage::StorageUint<VB, VL>;
    type Key = Uint<KB, KL>;
    type Value = Uint<VB, VL>;
}

pub(crate) trait Num: num_traits::NumOps + Ord + Sized + Copy{
    const ZERO: Self;
}

impl<const B: usize, const L: usize> Num for Uint<B, L> {
    const ZERO: Self = Self::ZERO;
}

trait Accessor {
    type Wrap: Num;
    fn get(&self) -> Self::Wrap;
    fn set(&mut self, value: Self::Wrap);
}

impl<const B: usize, const L: usize> Accessor for stylus_sdk::storage::StorageUint<B, L> {
    type Wrap = Uint<B, L>;

    fn get(&self) -> Self::Wrap {
        self.get()
    }

    fn set(&mut self, value: Self::Wrap) {
        self.set(value);
    }
}


impl<T: Size> Trace<T> {
    /// Pushes a (`key`, `value`) pair into a Trace160 so that it is
    /// stored as the checkpoint.
    ///
    /// Returns previous value and new value.
    ///
    /// IMPORTANT: Never accept `key` as a user input, since an arbitrary
    /// `type(uint96).max` key set will disable the library.
    pub fn push(
        &mut self,
        key: T::Key,
        value: T::Value,
    ) -> Result<(T::Value, T::Value), Error> {
        self._insert(key, value)
    }

    /// Returns the value in the first (oldest) checkpoint with key
    /// greater or equal than the search key, or zero if there is none.
    pub fn lower_lookup(&mut self, key: T::Key) -> T::Value {
        let len = self.length();
        let pos = self._lower_binary_lookup(key, U256::ZERO, len);
        if pos == len {
            T::Value::ZERO
        } else {
            self._unsafe_access_value(pos)
        }
    }

    /// Returns the value in the last (most recent) checkpoint with key
    /// lower or equal than the search key, or zero if there is none.
    pub fn upper_lookup(&mut self, key: T::Key) -> T::Value {
        let len = self.length();
        let pos = self._lower_binary_lookup(key, U256::ZERO, len);
        if pos == len {
            T::Value::ZERO
        } else {
            self._unsafe_access_value(pos)
        }
    }

    /// Returns the value in the last (most recent) checkpoint with key
    /// lower or equal than the search key, or zero if there is none.
    ///
    /// NOTE: This is a variant of {upperLookup} that is optimised to find
    /// "recent" checkpoint (checkpoints with high keys).
    pub fn upper_lookup_recent(&mut self, key: T::Key) -> T::Value {
        let len = self.length();

        let mut low = U256::ZERO;
        let mut high = len;
        if len > uint!(5_U256) {
            let mid = len - len.sqrt();
            if key < self._unsafe_access_key(mid) {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }

        let pos = self._upper_binary_lookup(key, low, high);

        if pos == U256::ZERO {
            T::Value::ZERO
        } else {
            self._unsafe_access_value(pos - uint!(1_U256))
        }
    }

    /// Returns the value in the most recent checkpoint, or zero if
    /// there are no checkpoints.
    pub fn latest(&mut self) -> T::Value {
        let pos = self.length();
        if pos == U256::ZERO {
            T::Value::ZERO
        } else {
            self._unsafe_access_value(pos - uint!(1_U256))
        }
    }

    /// Returns whether there is a checkpoint in the structure (i.e. it
    /// is not empty), and if so the key and value in the most recent
    /// checkpoint.
    pub fn latest_checkpoint(&self) -> (bool, T::Key, T::Value) {
        let pos = self.length();
        if pos == U256::ZERO {
            (false, T::Key::ZERO, T::Value::ZERO)
        } else {
            let checkpoint = self._unsafe_access(pos - uint!(1_U256));
            (true, checkpoint._key.load(), checkpoint._value.load())
        }
    }

    /// Returns the number of checkpoint.
    pub fn length(&self) -> U256 {
        // TODO#q: think how to retrieve U256 without conversion
        U256::from(self._checkpoints.len())
    }

    /// Returns checkpoint at given position.
    pub fn at(&self, pos: U32) -> Checkpoint<T> {
        unsafe { self._checkpoints.getter(pos).unwrap().into_raw() }
    }

    /// Pushes a (`key`, `value`) pair into an ordered list of
    /// checkpoints, either by inserting a new checkpoint, or by updating
    /// the last one.
    fn _insert(
        &mut self,
        key: T::Key,
        value: T::Value,
    ) -> Result<(T::Value, T::Value), Error> {
        let pos = self.length();
        if pos > U256::ZERO {
            let last = self._unsafe_access(pos - uint!(1_U256));
            let last_key = last._key.get();
            let last_value = last._value.get();

            // Checkpoint keys must be non-decreasing.
            if last_key > key {
                return Err(CheckpointUnorderedInsertion {}.into());
            }

            // Update or push new checkpoint
            if last_key > key {
                self._checkpoints
                    .setter(pos - uint!(1_U256))
                    .unwrap()
                    ._value
                    .set(value);
            } else {
                self.push(key, value)?;
            }
            Ok((last_value, value))
        } else {
            self.push(key, value)?;
            Ok((T::Value::ZERO, value))
        }
    }

    /// Return the index of the last (most recent) checkpoint with key
    /// lower or equal than the search key, or `high` if there is none.
    /// `low` and `high` define a section where to do the search, with
    /// inclusive `low` and exclusive `high`.
    ///
    /// WARNING: `high` should not be greater than the array's length.
    fn _upper_binary_lookup(
        &self,
        key: T::Key,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = low.average(high);
            if self._unsafe_access_key(mid) > key {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }
        high
    }

    /// Return the index of the first (oldest) checkpoint with key is
    /// greater or equal than the search key, or `high` if there is none.
    /// `low` and `high` define a section where to do the search, with
    /// inclusive `low` and exclusive `high`.
    ///
    /// WARNING: `high` should not be greater than the array's length.
    fn _lower_binary_lookup(
        &self,
        key: T::Key,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = low.average(high);
            if self._unsafe_access_key(mid) < key {
                low = mid + uint!(1_U256);
            } else {
                high = mid;
            }
        }
        high
    }

    /// Access on an element of the array without performing bounds check.
    /// The position is assumed to be within bounds.
    fn _unsafe_access(&self, pos: U256) -> Checkpoint<T> {
        // TODO#q: think how access it without bounds check
        unsafe { self._checkpoints.getter(pos).unwrap().into_raw() }
    }

    /// Access on a key
    fn _unsafe_access_key(&self, pos: U256) -> T::Key {
        // TODO#q: think how access it without bounds check
        let check_point =
            self._checkpoints.get(pos).expect("get checkpoint by index");
        check_point._key.get()
    }

    /// Access on a value
    fn _unsafe_access_value(&self, pos: U256) -> T::Value {
        // TODO#q: think how access it without bounds check
        let check_point =
            self._checkpoints.get(pos).expect("get checkpoint by index");
        check_point._value.get()
    }
}
