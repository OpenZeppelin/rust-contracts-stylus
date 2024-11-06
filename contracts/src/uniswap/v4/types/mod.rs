//! Module with data types for Uniswap V4 Hooks.
use alloy_primitives::{Signed, Uint, B256, I256};

mod liquidity;
mod pool_key;
mod swap_params;

pub use liquidity::ModifyLiquidityParams;
pub use pool_key::{Currency, PoolKey};
pub use swap_params::SwapParams;

/// Type representing Id of a Pool.
pub type PoolId = B256;

/// Type representing signed 24-bits integer.
pub type I24 = Signed<24, 1>;

/// Type representing unsigned 24-bits integer.
pub type U24 = Uint<24, 1>;

/// Type representing balance delta.
pub type BalanceDelta = I256;

/// Type representing before swap delta.
pub type BeforeSwapDelta = I256;
