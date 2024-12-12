#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{extensions::Erc20Metadata, Erc20};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc20Example {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub metadata: Erc20Metadata,
}

#[public]
#[inherit(Erc20, Erc20Metadata)]
impl Erc20Example {
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc20._mint(account, value)?;
        Ok(())
    }
}
