#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{extensions::IErc721Burnable, Erc721};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc721ConsecutiveExample {
        #[borrow]
        Erc721 erc721;
    }
}

// TODO#q: add consecutive errors

#[external]
#[inherit(Erc721)]
impl Erc721ConsecutiveExample {
    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721.burn(token_id)?;
        Ok(())
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721._mint(to, token_id)?;

        Ok(())
    }

    // TODO#q: add consecutive implementation
}
