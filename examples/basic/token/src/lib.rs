#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    self, extensions::Erc20Metadata, Erc20,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc20Example {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    metadata: Erc20Metadata,
}

#[public]
#[inherit(Erc20, Erc20Metadata)]
impl Erc20Example {
    #[constructor]
    fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }

    fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), erc20::Error> {
        self.erc20._mint(account, value)
    }
}
