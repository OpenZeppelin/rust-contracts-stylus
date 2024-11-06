#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, Bytes, FixedBytes, I128, U256};
use openzeppelin_stylus::uniswap::v4::{
    hooks::Error, BalanceDelta, BeforeSwapDelta, IHooks, ModifyLiquidityParams,
    Permissions, PoolId, PoolKey, SwapParams, U24,
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct UniswapV4HooksExample {
        mapping(PoolId => uint256) beforeSwapCount;
        mapping(PoolId => uint256) afterSwapCount;

        mapping(PoolId => uint256) beforeAddLiquidityCount;
        mapping(PoolId => uint256) beforeRemoveLiquidityCount;
    }
}

#[public]
impl IHooks for UniswapV4HooksExample {
    #[selector(name = "getHookPermissions")]
    fn get_hook_permissions(&self) -> Permissions {
        Permissions {
            beforeInitialize: false,
            afterInitialize: false,
            beforeAddLiquidity: true,
            afterAddLiquidity: false,
            beforeRemoveLiquidity: true,
            afterRemoveLiquidity: false,
            beforeSwap: true,
            afterSwap: true,
            beforeDonate: false,
            afterDonate: false,
            beforeSwapReturnDelta: false,
            afterSwapReturnDelta: false,
            afterAddLiquidityReturnDelta: false,
            afterRemoveLiquidityReturnDelta: false,
        }
    }

    #[selector(name = "beforeSwap")]
    fn before_swap(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: SwapParams,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, BeforeSwapDelta, U24), Error> {
        let id = key.into();
        let counter = self.beforeSwapCount.getter(id).get();
        let counter = counter
            .checked_add(U256::from(1))
            .expect("should not exceed `U256::MAX`");
        self.beforeSwapCount.setter(id).set(counter);
        todo!()
    }

    #[selector(name = "afterSwap")]
    fn after_swap(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: SwapParams,
        delta: BalanceDelta,
        hook_data: Bytes,
    ) -> Result<(FixedBytes<4>, I128), Error> {
        let id = key.into();
        let counter = self.afterSwapCount.getter(id).get();
        let counter = counter
            .checked_add(U256::from(1))
            .expect("should not exceed `U256::MAX`");
        self.afterSwapCount.setter(id).set(counter);
        todo!()
    }

    #[selector(name = "beforeAddLiquidity")]
    fn before_add_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Error> {
        let id = key.into();
        let counter = self.beforeAddLiquidityCount.getter(id).get();
        let counter = counter
            .checked_add(U256::from(1))
            .expect("should not exceed `U256::MAX`");
        self.beforeAddLiquidityCount.setter(id).set(counter);
        todo!()
    }

    #[selector(name = "beforeRemoveLiquidity")]
    fn before_remove_liquidity(
        &mut self,
        sender: Address,
        key: PoolKey,
        params: ModifyLiquidityParams,
        hook_data: Bytes,
    ) -> Result<FixedBytes<4>, Error> {
        let id = key.into();
        let counter = self.beforeRemoveLiquidityCount.getter(id).get();
        let counter = counter
            .checked_add(U256::from(1))
            .expect("should not exceed `U256::MAX`");
        self.beforeRemoveLiquidityCount.setter(id).set(counter);
        todo!()
    }
}
