#![cfg_attr(not(test), no_main)]
extern crate alloc;

use core::borrow;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use stylus_sdk::prelude::{entrypoint, public, storage};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{ Erc20Metadata,Erc4626, IERC4626},
        Erc20, IErc20,
    },
    utils::{introspection::erc165::IErc165, Pausable},
};


#[entrypoint]
#[storage]
struct Erc4262xample {
    #[borrow]
    pub erc20: Erc20,
     #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub erc4626: Erc4626,
}


#[public]
#[inherit(Erc20, Erc20Metadata, Erc4626)]
impl Erc4262xample {
    // Add token minting feature.
}
