#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        extensions::consecutive::{Erc721Consecutive, Error},
        Erc721,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc721ConsecutiveExample {
    #[borrow]
    erc721_consecutive: Erc721Consecutive,
}

#[public]
#[inherit(Erc721Consecutive)]
impl Erc721ConsecutiveExample {
    fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._burn(token_id)
    }

    fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._mint(to, token_id)
    }

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(interface_id)
    }
}
