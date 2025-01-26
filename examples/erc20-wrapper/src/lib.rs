#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
        extensions::{Erc20Metadata, Erc20Wrapper,IERC20Wrapper},
        Erc20,
    };
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc20WrapperExample {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub wrapper: Erc20Wrapper,
}

#[public]
#[inherit(Erc20, Erc20Metadata)]
impl Erc20WrapperExample {
    fn underlying(&self) -> Address {
         self.wrapper.underlying()
    }

    fn deposit_to(
        &mut self,
        account: Address,
        value: U256
    ) -> Result<bool, Vec<u8>> {
        Ok(self.wrapper.deposit_to(account, value, &mut self.erc20)?)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256
    ) -> Result<bool, Vec<u8>> {
        Ok(self.wrapper.withdraw_to(account, value, &mut self.erc20)?)
    }
}
