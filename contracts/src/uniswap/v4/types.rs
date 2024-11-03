//! Module with data types for Uniswap V4 Hooks.
use alloy_primitives::{Address, Signed, Uint, I256};
use alloy_sol_types::sol;

// TODO: newer alloy's versions have `I24`.
pub type I24 = Signed<24, 1>;
// TODO: newer alloy's versions have `U24`.
pub type U24 = Uint<24, 1>;

pub type BalanceDelta = I256;
pub type BeforeSwapDelta = I256;

pub type Currency = Address;

sol! {
    struct ModifyLiquidityParams {
        /// the lower and upper tick of the position
        int24 tickLower;
        int24 tickUpper;
        /// how to modify the liquidity
        int256 liquidityDelta;
        //// a value to set if you want unique liquidity positions at the same range
        bytes32 salt;
    }

}

sol! {
    struct SwapParams {
        /// Whether to swap token0 for token1 or vice versa
        bool zeroForOne;
        /// The desired input amount if negative (exactIn), or the desired output amount if positive (exactOut)
        int256 amountSpecified;
        /// The sqrt price at which, if reached, the swap will stop executing
        uint160 sqrtPriceLimitX96;
    }
}

sol! {
    /// Returns the key for identifying a pool.
    struct PoolKey {
        /// The lower currency of the pool, sorted numerically.
        // TODO
        address currency0;
        /// The higher currency of the pool, sorted numerically.
        // TODO
        address currency1;
        /// The pool LP fee, capped at 1_000_000.
        /// If the highest bit is 1, the pool has a dynamic fee and
        /// must be exactly equal to 0x800000.
        uint24 fee;
        /// Ticks that involve positions must be a multiple of tick spacing
        int24 tickSpacing;
        /*
        /// @notice The hooks of the pool
        IHooks hooks;
        */
    }
}

pub use interface::IHooks;
#[allow(missing_docs)]
mod interface {
    stylus_sdk::stylus_proc::sol_interface! {
             interface IHooks {
    /*
                 function beforeInitialize(address sender, PoolKey calldata key, uint160 sqrtPriceX96) external returns (bytes4);

                 function afterInitialize(address sender, PoolKey calldata key, uint160 sqrtPriceX96, int24 tick)
                     external
                     returns (bytes4);

        function beforeAddLiquidity(
             address sender,
             PoolKey calldata key,
             IPoolManager.ModifyLiquidityParams calldata params,
             bytes calldata hookData
         ) external returns (bytes4);

        function afterAddLiquidity(
             address sender,
             PoolKey calldata key,
             IPoolManager.ModifyLiquidityParams calldata params,
             BalanceDelta delta,
             BalanceDelta feesAccrued,
             bytes calldata hookData
         ) external returns (bytes4, BalanceDelta);

        function beforeRemoveLiquidity(
             address sender,
             PoolKey calldata key,
             IPoolManager.ModifyLiquidityParams calldata params,
             bytes calldata hookData
         ) external returns (bytes4);

        function afterRemoveLiquidity(
             address sender,
             PoolKey calldata key,
             IPoolManager.ModifyLiquidityParams calldata params,
             BalanceDelta delta,
             BalanceDelta feesAccrued,
             bytes calldata hookData
         ) external returns (bytes4, BalanceDelta);

        function beforeSwap(
             address sender,
             PoolKey calldata key,
             IPoolManager.SwapParams calldata params,
             bytes calldata hookData
         ) external returns (bytes4, BeforeSwapDelta, uint24);

        function afterSwap(
             address sender,
             PoolKey calldata key,
             IPoolManager.SwapParams calldata params,
             BalanceDelta delta,
             bytes calldata hookData
         ) external returns (bytes4, int128);

        function beforeDonate(
             address sender,
             PoolKey calldata key,
             uint256 amount0,
             uint256 amount1,
             bytes calldata hookData
         ) external returns (bytes4);

        function afterDonate(
             address sender,
             PoolKey calldata key,
             uint256 amount0,
             uint256 amount1,
             bytes calldata hookData
         ) external returns (bytes4);
         */
         }

         }
}
