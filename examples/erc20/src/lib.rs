#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use alloy_primitives::{Address, U256};
use contracts::{
    derive_erc20_burnable,
    erc20::{extensions::Metadata, Error, ERC20},
};
use stylus_sdk::{
    msg,
    prelude::{entrypoint, external, sol_storage},
};

const DECIMALS: u8 = 10;

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC20 erc20;
        #[borrow]
        Metadata metadata;
    }
}

#[external]
#[inherit(ERC20, Metadata)]
impl Token {
    derive_erc20_burnable!();

    pub fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }

    // Overrides the default [`Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    pub fn decimals(&self) -> u8 {
        DECIMALS
    }
}
