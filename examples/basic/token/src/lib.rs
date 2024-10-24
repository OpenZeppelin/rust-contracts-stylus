#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{extensions::Erc20Metadata, Erc20};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc20Example {
        #[borrow]
        Erc20 erc20;
        #[borrow]
        Erc20Metadata metadata;
    }
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
