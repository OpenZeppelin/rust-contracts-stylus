#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::token::erc20::extensions::Erc4626;
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc4626Example {
    #[borrow]
    pub erc4626: Erc4626,
}

#[public]
#[inherit(Erc4626)]
impl Erc4626Example {}
