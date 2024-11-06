use alloy_sol_types::sol;

sol! {
    /// Struct representing modify liquidity operation parameters.
    struct ModifyLiquidityParams {
        /// The lower tick of the position.
        int24 tickLower;
        /// The upper tick of the position.
        int24 tickUpper;
        /// How to modify the liquidity.
        int256 liquidityDelta;
        /// A value to set if you want unique liquidity positions
        /// at the same range.
        bytes32 salt;
    }

}
