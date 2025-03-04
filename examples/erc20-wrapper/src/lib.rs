#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::{Erc20Wrapper, IErc20Wrapper},
    Erc20,
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc20WrapperExample {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub erc20_wrapper: Erc20Wrapper,
}

#[public]
#[inherit(Erc20)]
impl Erc20WrapperExample {
    fn underlying(&self) -> Address {
        self.erc20_wrapper.underlying()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        Ok(self.erc20_wrapper.deposit_for(account, value, &mut self.erc20)?)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        Ok(self.erc20_wrapper.withdraw_to(account, value, &mut self.erc20)?)
    }
}
