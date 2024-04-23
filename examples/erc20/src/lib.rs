#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use contracts::{
    erc20,
    erc20::{extensions::Metadata, ERC20},
    utils::pausable::{IPausable, Pausable},
};
use erc20_proc::{
    ICapped, IERC20Burnable, IERC20Capped, IERC20Pausable, IERC20Storage,
    IERC20Virtual, IPausable, IERC20,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};
const DECIMALS: u8 = 10;
use contracts::{
    erc20::{extensions::burnable::IERC20Burnable, IERC20},
    erc20_burnable_impl,
    utils::capped::{Capped, ICapped},
};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        BurnableCappedPausableERC20 erc20;
        #[borrow]
        Metadata metadata;
    }

    #[derive(IERC20Storage, IERC20, IERC20Virtual, IERC20Burnable, IPausable, ICapped)]
    struct BurnableCappedPausableERC20 {
        CappedPausableERC20 erc20;
    }

    #[derive(IERC20Storage, IERC20, IPausable, IERC20Capped)]
    struct CappedPausableERC20 {
        PausableERC20 erc20;
        Capped capped;
    }

    #[derive(IERC20Storage, IERC20, IERC20Pausable)]
    struct PausableERC20 {
        ERC20 erc20;
        Pausable pausable
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
