use alloy_sol_types::sol;

sol! {
    /// Struct representing swap parameters.
    struct SwapParams {
        /// Whether to swap token0 for token1 or vice versa.
        bool zeroForOne;
        /// The desired input amount if negative (exactIn),
        /// or the desired output amount if positive (exactOut).
        int256 amountSpecified;
        /// The sqrt price at which, if reached, the swap will stop executing.
        uint160 sqrtPriceLimitX96;
    }
}
