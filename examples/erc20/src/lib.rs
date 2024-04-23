#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use contracts::{
    erc20::{
        extensions::{burnable::IERC20Burnable, Metadata},
        ERC20,
    },
    erc20_burnable_impl,
};
use erc20_proc::{IERC20Burnable, IERC20Storage, IERC20Virtual, IERC20};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

const DECIMALS: u8 = 10;

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC20Burnable erc20;
        #[borrow]
        Metadata metadata;
    }

    #[derive(IERC20Storage, IERC20Virtual, IERC20, IERC20Burnable)]
    struct ERC20Burnable {
        ERC20 erc20;
    }
}

#[external]
#[inherit(Metadata)]
impl Token {
    // This macro implements ERC20Burnable functions -- `burn` and `burn_from`.
    // Expects an `ERC20 erc20` as a field of `Token`.
    erc20_burnable_impl!();

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
