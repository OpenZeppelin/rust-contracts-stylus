//! Contract module for checkpointing values as they
//! change at different points in time, and later looking up past values by
//! block number. See {Votes} as an example. To create a history of checkpoints
//! define a variable type `Checkpoints.Trace*` in your contract, and store a
//! new checkpoint for the current transaction block using the {push} function.
use alloy_primitives::{uint, Uint, U256, U32};
use alloy_sol_types::sol;
use stylus_proc::{sol_storage, SolidityError};
use stylus_sdk::prelude::StorageType;

use crate::utils::math::{average, sqrt};

type U96 = Uint<96, 2>;
type U160 = Uint<160, 3>;

sol! {
    /// A value was attempted to be inserted on a past checkpoint.
    #[derive(Debug)]
    error CheckpointUnorderedInsertion();
}

#[derive(SolidityError, Debug)]
pub enum Error {
    CheckpointUnorderedInsertion(CheckpointUnorderedInsertion),
}

sol_storage! {
    struct Trace160 {
        Checkpoint160[] _checkpoints;
    }

    struct Checkpoint160 {
        uint96 _key;
        uint160 _value;
    }
}

impl Trace160 {
    /**
     * @dev Pushes a (`key`, `value`) pair into a Trace160 so that it is
     * stored as the checkpoint.
     *
     * Returns previous value and new value.
     *
     * IMPORTANT: Never accept `key` as a user input, since an arbitrary
     * `type(uint96).max` key set will disable the library.
     */
    pub fn push(
        &mut self,
        key: U96,
        value: U160,
    ) -> Result<(U160, U160), Error> {
        self._insert(key, value)
    }

    /**
     * @dev Returns the value in the first (oldest) checkpoint with key
     * greater or equal than the search key, or zero if there is none.
     */
    pub fn lower_lookup(&mut self, key: U96) -> U160 {
        let len = self.length();
        let pos = self._lower_binary_lookup(key, U256::ZERO, len);
        if pos == len {
            U160::ZERO
        } else {
            self._unsafe_access_value(pos)
        }
    }

    /**
     * @dev Returns the value in the last (most recent) checkpoint with key
     * lower or equal than the search key, or zero if there is none.
     */
    pub fn upper_lookup(&mut self, key: U96) -> U160 {
        let len = self.length();
        let pos = self._lower_binary_lookup(key, U256::ZERO, len);
        if pos == len {
            U160::ZERO
        } else {
            self._unsafe_access_value(pos)
        }
    }

    /**
     * @dev Returns the value in the last (most recent) checkpoint with key
     * lower or equal than the search key, or zero if there is none.
     *
     * NOTE: This is a variant of {upperLookup} that is optimised to find
     * "recent" checkpoint (checkpoints with high keys).
     */
    pub fn upper_lookup_recent(&mut self, key: U96) -> U160 {
        let len = self.length();
        // TODO#q: use uint!(1_U256);

        let mut low = U256::ZERO;
        let mut high = len;
        if len > U256::from(5) {
            // NOTE#q: square root from `ruint` crate works just with std
            let mid = len - sqrt(len);
            if key < self._unsafe_access_key(mid) {
                high = mid;
            } else {
                low = mid + U256::from(1);
            }
        }

        let pos = self._upper_binary_lookup(key, low, high);

        if pos == U256::ZERO {
            U160::ZERO
        } else {
            self._unsafe_access_value(pos - U256::from(1))
        }
    }

    /**
     * @dev Returns the value in the most recent checkpoint, or zero if
     * there are no checkpoints.
     */
    pub fn latest(&mut self) -> U160 {
        let pos = self.length();
        if pos == U256::ZERO {
            U160::ZERO
        } else {
            self._unsafe_access_value(pos - U256::from(1))
        }
    }

    /**
     * @dev Returns whether there is a checkpoint in the structure (i.e. it
     * is not empty), and if so the key and value in the most recent
     * checkpoint.
     */
    pub fn latest_checkpoint(&self) -> (bool, U96, U160) {
        let pos = self.length();
        if pos == U256::ZERO {
            (false, U96::ZERO, U160::ZERO)
        } else {
            let checkpoint = self._unsafe_access(pos - U256::from(1));
            (true, checkpoint._key.load(), checkpoint._value.load())
        }
    }

    /**
     * @dev Returns the number of checkpoint.
     */
    pub fn length(&self) -> U256 {
        // TODO#q: think how to retrieve U256 without conversion
        U256::from(self._checkpoints.len())
    }

    /**
     * @dev Returns checkpoint at given position.
     */
    pub fn at(&self, pos: U32) -> Checkpoint160 {
        unsafe { self._checkpoints.getter(pos).unwrap().into_raw() }
    }

    /**
     * @dev Pushes a (`key`, `value`) pair into an ordered list of
     * checkpoints, either by inserting a new checkpoint, or by updating
     * the last one.
     */
    fn _insert(
        &mut self,
        key: U96,
        value: U160,
    ) -> Result<(U160, U160), Error> {
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
            Ok((U160::ZERO, value))
        }
    }

    /**
     * @dev Return the index of the last (most recent) checkpoint with key
     * lower or equal than the search key, or `high` if there is none.
     * `low` and `high` define a section where to do the search, with
     * inclusive `low` and exclusive `high`.
     *
     * WARNING: `high` should not be greater than the array's length.
     */
    fn _upper_binary_lookup(
        &self,
        key: U96,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = average(low, high);
            if self._unsafe_access_key(mid) > key {
                high = mid;
            } else {
                low = mid + uint!(1_U256);
            }
        }
        high
    }

    /**
     * @dev Return the index of the first (oldest) checkpoint with key is
     * greater or equal than the search key, or `high` if there is none.
     * `low` and `high` define a section where to do the search, with
     * inclusive `low` and exclusive `high`.
     *
     * WARNING: `high` should not be greater than the array's length.
     */
    fn _lower_binary_lookup(
        &self,
        key: U96,
        mut low: U256,
        mut high: U256,
    ) -> U256 {
        while low < high {
            let mid = average(low, high);
            if self._unsafe_access_key(mid) < key {
                low = mid + uint!(1_U256);
            } else {
                high = mid;
            }
        }
        high
    }

    /**
     * @dev Access an element of the array without performing bounds check.
     * The position is assumed to be within bounds.
     */
    fn _unsafe_access(&self, pos: U256) -> Checkpoint160 {
        // TODO#q: think how access it without bounds check
        unsafe { self._checkpoints.getter(pos).unwrap().into_raw() }
    }

    /// Access on a key
    fn _unsafe_access_key(&self, pos: U256) -> U96 {
        // TODO#q: think how access it without bounds check
        let check_point =
            self._checkpoints.get(pos).expect("get checkpoint by index");
        check_point._key.get()
    }

    /// Access on a value
    fn _unsafe_access_value(&self, pos: U256) -> U160 {
        // TODO#q: think how access it without bounds check
        let check_point =
            self._checkpoints.get(pos).expect("get checkpoint by index");
        check_point._value.get()
    }
}
