#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::extensions::{
    pausable::Error, Erc721Pausable,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc721Example {
        #[borrow]
        Erc721Pausable erc721;
    }
}

#[external]
#[inherit(Erc721Pausable)]
impl Erc721Example {
    pub fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        self.erc721._burn(token_id)
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721._mint(to, token_id)
    }

    pub fn paused(&self) -> bool {
        self.erc721.pausable.paused()
    }

    pub fn pause(&mut self) -> Result<(), Error> {
        Ok(self.erc721.pausable.pause()?)
    }

    pub fn unpause(&mut self) -> Result<(), Error> {
        Ok(self.erc721.pausable.unpause()?)
    }
}
