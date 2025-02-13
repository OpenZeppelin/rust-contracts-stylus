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
        first_consecutive_id: U96,
        max_batch_size: U96,
    ) -> Result<(), Vec<u8>> {
        self.erc721_consecutive._first_consecutive_id.set(first_consecutive_id);
        self.erc721_consecutive._max_batch_size.set(max_batch_size);
        for (&receiver, &amount) in receivers.iter().zip(amounts.iter()) {
            self.erc721_consecutive._mint_consecutive(receiver, amount)?;
        }
        Ok(())
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._burn(token_id)
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._mint(to, token_id)
    }
}
