#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    token::erc721::extensions::consecutive::{Erc721Consecutive, Error},
    utils::structs::checkpoints::U96,
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

    pub fn init(
        &mut self,
        receivers: Vec<Address>,
        batches: Vec<U256>,
    ) -> Result<(), Error> {
        let len = batches.len();
        for i in 0..len {
            let receiver = receivers[i];
            let batch = batches[i];
            let _ = self
                .erc721_consecutive
                ._mint_consecutive(receiver, U96::from(batch))?;
        }
        self.erc721_consecutive._stop_mint_consecutive();
        Ok(())
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721_consecutive._mint(to, token_id)
    }
}
