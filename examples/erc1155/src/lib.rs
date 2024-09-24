#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use openzeppelin_stylus::{token::erc1155::Erc1155, utils::Pausable};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc1155Example {
        #[borrow]
        Erc1155 erc1155;
        #[borrow]
        Pausable pausable;
    }
}

#[public]
#[inherit(Erc1155, Pausable)]
impl Erc1155Example {}
