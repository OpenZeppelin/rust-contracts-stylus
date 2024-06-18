//! Contract module for checkpointing values as they
//! change at different points in time, and later looking up past values by
//! block number. See {Votes} as an example. To create a history of checkpoints
//! define a variable type `Checkpoints.Trace*` in your contract, and store a
//! new checkpoint for the current transaction block using the {push} function.
use alloy_primitives::{Uint, U256, U32};
use alloy_sol_types::sol;
use stylus_proc::sol_storage;
type U96 = Uint<96, 2>;
type U160 = Uint<160, 3>;

sol! {
    /// A value was attempted to be inserted on a past checkpoint.
    error CheckpointUnorderedInsertion();
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
    pub fn push(&mut self, key: U96, value: U160) -> (U160, U160) {
        self._insert(key, value)
    }

    /**
     * @dev Pushes a (`key`, `value`) pair into an ordered list of
     * checkpoints, either by inserting a new checkpoint, or by updating
     * the last one.
     */
    fn _insert(&mut self, key: U96, value: U160) -> (U160, U160) {
        todo!()
    }
}
