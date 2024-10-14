#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::{Erc20Metadata, IErc20Metadata},
    Erc20,
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc20Example {
        #[borrow]
        Erc20 erc20;
    }
}

#[public]
#[inherit(Erc20)]
impl Erc20Example {
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc20._mint(account, value)?;
        Ok(())
    }

    pub fn name(&self) -> String {
        self.erc20.name()
    }

    pub fn symbol(&self) -> String {
        self.erc20.symbol()
    }

    pub fn decimals(&self) -> u8 {
        self.erc20.decimals()
    }
}
