use alloy_primitives::keccak256;
use alloy_sol_types::{sol, SolValue};

use super::PoolId;

sol! {
    /// The currency data type.
    type Currency is address;

    /// Returns the key for identifying a pool.
    struct PoolKey {
        /// The lower currency of the pool, sorted numerically.
        Currency currency0;
        /// The higher currency of the pool, sorted numerically.
        Currency currency1;
        /// The pool LP fee, capped at 1_000_000.
        /// If the highest bit is 1, the pool has a dynamic fee and
        /// must be exactly equal to 0x800000.
        uint24 fee;
        /// Ticks that involve positions must be a multiple of tick spacing.
        int24 tickSpacing;
        /*
        // TODO
        // The hooks of the pool
        // IHooks hooks;
        */
    }
}

impl From<PoolKey> for PoolId {
    fn from(value: PoolKey) -> Self {
        let encoded = PoolKey::abi_encode(&value);
        keccak256(encoded)
    }
}
