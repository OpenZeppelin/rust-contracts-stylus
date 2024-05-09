#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

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
impl Token {}
