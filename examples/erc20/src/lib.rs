#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use contracts::{
    erc20::{
        extensions::{burnable::IERC20Burnable, Metadata},
        ERC20, IERC20,
    },
    erc20_burnable_impl, erc20_impl,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

const DECIMALS: u8 = 10;
use contracts::{
    erc20::extensions::pausable::ERC20Pausable, utils::pausable::Pausable,
};

pub type ERC20Custom = ERC20Pausable<ERC20, Pausable>;
// impl IERC20Burnable for ERC20Custom {}

sol_storage! {
    #[entrypoint]
    struct Token {
        ERC20 erc20;
        #[borrow]
        Metadata metadata;
    }
}

#[external]
#[inherit(Metadata)]
impl Token {
    // This macro implements ERC20Burnable functions -- `burn` and `burn_from`.
    // Expects an `ERC20 erc20` as a field of `Token`.
    erc20_impl!();

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
