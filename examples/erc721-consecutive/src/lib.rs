#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::extensions::consecutive::{
    Erc721Consecutive, Error,
};
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    struct Erc721ConsecutiveExample {
        #[borrow]
        Erc721Consecutive erc721_consecutive;
    }
}

#[external]
#[inherit(Erc721Consecutive)]
impl Erc721ConsecutiveExample {
    pub fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._burn(token_id)
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._mint(to, token_id)
    }
}
