#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{aliases::U96, Address, U256};
use openzeppelin_stylus::token::erc721::extensions::consecutive::{
    Erc721Consecutive, Error,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc721ConsecutiveExample {
    #[borrow]
    pub erc721_consecutive: Erc721Consecutive,
}

#[public]
#[inherit(Erc721Consecutive)]
impl Erc721ConsecutiveExample {
    #[constructor]
    fn constructor(
        &mut self,
        receivers: Vec<Address>,
        amounts: Vec<U96>,
        firstConsecutiveId: U96,
        maxBatchSize: U96,
    ) {
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._burn(token_id)
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._mint(to, token_id)
    }
}
