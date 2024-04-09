#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

// Import required by ERC20Burnable extension.
use alloy_primitives::{Address, U256};
// Import Metadata extension.
use contracts::erc20::extensions::Metadata;
// Import ERC20 token and its Errors.
use contracts::erc20::{Error, ERC20};
// Import implementation of ERC20Burnable extension.
use contracts::impl_erc20_burnable;
// Import required by ERC20Burnable extension.
use stylus_sdk::msg;
// Stylus imports to build a smart contract.
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
    // This macro implements ERC20Burnable functions -- `burn` and `burn_from`.
    // Uses `erc20` Token's attribute.
    impl_erc20_burnable!();

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
