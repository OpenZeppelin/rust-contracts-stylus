#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{extensions::Erc721Wrapper, Erc721};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc721WrapperExample {
    #[borrow]
    pub erc721: Erc721,
    #[borrow]
    pub wrapper: Erc721Wrapper,
}

#[public]
#[inherit(Erc721)]
impl Erc721WrapperExample {
    fn underlying(&self) -> Address {
        self.wrapper.underlying()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        values: Vec<U256>,
    ) -> Result<bool, Vec<u8>> {
        Ok(self.wrapper.deposit_for(account, values, &mut self.erc721)?)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        values: Vec<U256>,
    ) -> Result<bool, Vec<u8>> {
        Ok(self.wrapper.withdraw_to(account, values, &mut self.erc721)?)
    }
}
