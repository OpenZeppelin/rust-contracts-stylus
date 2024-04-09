#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use contracts::erc20::{extensions::Metadata, ERC20};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

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
    pub fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }

    pub fn decimals(&self) -> u8 {
        DECIMALS
    }
}
