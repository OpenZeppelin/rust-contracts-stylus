#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256, U8};
use openzeppelin_stylus::token::erc20::{
    extensions::{wrapper, Erc20Wrapper, IErc20Wrapper},
    Erc20,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc20WrapperExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    erc20_wrapper: Erc20Wrapper,
}

#[public]
#[inherit(Erc20)]
impl Erc20WrapperExample {
    #[constructor]
    fn constructor(
        &mut self,
        underlying_token: Address,
        decimals: U8,
    ) -> Result<(), wrapper::Error> {
        self.erc20_wrapper.constructor(underlying_token)?;
        self.erc20_wrapper.underlying_decimals.set(decimals);
        Ok(())
    }

    fn underlying(&self) -> Address {
        self.erc20_wrapper.underlying()
    }

    fn decimals(&self) -> U8 {
        self.erc20_wrapper.decimals()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, wrapper::Error> {
        self.erc20_wrapper.deposit_for(account, value, &mut self.erc20)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, wrapper::Error> {
        self.erc20_wrapper.withdraw_to(account, value, &mut self.erc20)
    }
}
