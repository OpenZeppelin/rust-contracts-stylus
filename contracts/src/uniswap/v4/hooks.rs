//! Uniswap V4 Hooks Interface.
//!
//! V4 decides whether to invoke specific hooks by inspecting the least
//! significant bits of the address that the hooks contract is deployed to.
//! For example, a hooks contract deployed to address:
//! 0x0000000000000000000000000000000000002400
//! has the lowest bits '10 0100 0000 0000' which would cause
//! the 'before initialize' and 'after add liquidity' hooks to be used.
//!
//! See the Hooks library for the full spec.
//!
//! Should only be callable by the v4 PoolManager.

use alloy_primitives::{Address, Bytes, FixedBytes, I128, U160, U256};
use stylus_sdk::stylus_proc::SolidityError;

use crate::uniswap::v4::{
    BalanceDelta, BeforeSwapDelta, ModifyLiquidityParams, PoolKey, SwapParams,
    I24, U24,
};

/// An Uniswap V4 Hook error.
#[derive(SolidityError, Debug)]
pub enum Error {}

/// Uniswap V4 Hooks Interface.
pub trait IHooks {
    /// The hook called before the state of a pool is initialized.
    ///
    /// Returns the function selector for the hook.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` -The initial msg::sender() for the initialize call.
    /// * `key` - The key for the pool being initialized.
    /// * `sqrt_price_x96` - The sqrt(price) of the pool as a Q64.96.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn before_initialize(
        &mut self,
        sender: Address,
        key: PoolKey,
        sqrt_price_x96: U160,
    ) -> Result<FixedBytes<4>, Error>;

    /// The hook called after the state of a pool is initialized.
    ///
    /// Returns the function selector for the hook.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the initialize call.
    /// * `key` - The key for the pool being initialized.
    /// * `sqrt_price_x96` -  The sqrt(price) of the pool as a Q64.96.
    /// * `tick` - The current tick after the state of a pool is initialized.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn after_initialize(
        &mut self,
        sender: Address,
        key: PoolKey,
        sqrt_price_x96: U160,
        tick: I24,
    ) -> Result<FixedBytes<4>, Error>;

    /// The hook called before liquidity is added.
    ///
    /// Returns the function selector for the hook.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the add liquidity call.
    /// * `key` - The key for the pool.
    /// * `params` - The parameters for adding liquidity.
    /// *` hook_data` - Arbitrary data handed into the Pool Manager by the
    ///    liquidity provider to be passed on to the hook.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn before_add_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Error>;

    /// The hook called after liquidity is added
    ///
    /// Returns tuple containing the function selector for the hook, and
    /// the hook's delta in token0 and token1.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the add liquidity call.
    /// * `key` - The key for the pool.
    /// * `params` - The parameters for adding liquidity.
    /// * `delta` - The caller's balance delta after adding liquidity; the sum
    ///   of principal delta, fees accrued, and hook delta.
    /// * `fees_accrued` - The fees accrued since the last time fees were
    ///   collected from this position.
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the
    ///   liquidity provider to be passed on to the hook.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn after_add_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        delta: BalanceDelta,
        fees_accrued: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BalanceDelta), Error>;

    /// The hook called before liquidity is removed.
    ///
    /// Returns the function selector for the hook.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the remove liquidity call.
    /// * `key` -  The key for the pool.
    /// * `params` - The parameters for removing liquidity.
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the
    ///   liquidity provider to be be passed on to the hook.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn before_remove_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Error>;

    /// The hook called after liquidity is removed.
    ///
    /// Returns tuple containing the function selector for the hook, and
    /// the hook's delta in token0 and token1.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the remove liquidity call.
    /// * `key` - The key for the pool.
    /// * `params` - The parameters for removing liquidity.
    /// * `delta` - The caller's balance delta after removing liquidity; the sum
    ///   of principal delta, fees accrued, and hook delta.
    /// * `fees_accrued` - The fees accrued since the last time fees were
    ///   collected from this position.
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the
    ///   liquidity provider to be be passed on to the hook.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn after_remove_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        delta: BalanceDelta,
        fees_accrued: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BalanceDelta), Error>;

    /// The hook called before a swap.
    ///
    /// Returns tuple containing the function selector for the hook, and
    /// the hook's delta in specified and unspecified currencies, and
    /// optionally override the lp fee, only used if three conditions are met:
    /// 1. the Pool has a dynamic fee,
    /// 2. the value's 2nd highest bit is set (23rd bit, 0x400000), and
    /// 3. the value is less than or equal to the maximum fee (1 million).
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the swap call.
    /// * `key` - The key for the pool.
    /// * `params` - The parameters for the swap.
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the
    ///   swapper to be be passed on to the hook.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn before_swap(
        sender: Address,
        key: PoolKey,
        params: SwapParams,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Error>;

    /// The hook called after a swap.
    ///
    /// Returns the function selector for the hook, and
    /// the hook's delta in unspecified currency.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the swap call.
    /// * `key` - The key for the pool.
    /// * `params` - The parameters for the swap.
    /// * `delta` - The amount owed to the caller (positive), or owed to the
    ///   pool (negative).
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the
    ///   swapper to be be passed on to the hook.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn after_swap(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: SwapParams,
        delta: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, I128), Error>;

    /// The hook called before donate.
    ///
    /// Returns the function selector for the hook.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the donate call.
    /// * `key` - The key for the pool.
    /// * `amount0` - The amount of token0 being donated.
    /// * `amount1` - The amount of token1 being donated.
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the donor
    ///   to be be passed on to the hook
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn before_donate(
        &mut self,
        sender: Address,
        key: PoolKey,
        amount0: U256,
        amount1: U256,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Error>;

    /// The hook called after donate
    ///
    /// Returns the function selector for the hook.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - The initial msg::sender() for the donate call.
    /// * `key` - The key for the pool.
    /// * `amount0` The amount of token0 being donated.
    /// * `amount1` - The amount of token1 being donated.
    /// * `hook_data` - Arbitrary data handed into the Pool Manager by the donor
    ///   to be be passed on to the hook
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn after_donate(
        &mut self,
        sender: Address,
        key: PoolKey,
        amount0: U256,
        amount1: U256,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Error>;
}
