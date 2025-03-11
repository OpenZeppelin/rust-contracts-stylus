#![cfg_attr(not(test), no_main)]
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
