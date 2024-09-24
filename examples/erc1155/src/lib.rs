#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc1155::Erc1155;
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Erc1155Example {
        #[borrow]
        Erc1155 erc1155;
    }
}

#[public]
#[inherit(Erc1155)]
impl Erc1155Example {}
