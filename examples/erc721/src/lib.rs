#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::Address;
use stylus_sdk::alloy_sol_types::private::U256;
use contracts::erc721::ERC721;
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC721 erc721;
    }
}

#[external]
#[inherit(ERC721)]
impl Token {
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), contracts::erc721::Error> {
        self.erc721._mint(to, token_id)
    }
}
