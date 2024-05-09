#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use contracts::erc721::{extensions::ERC721Metadata, ERC721};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC721 erc721;
        #[borrow]
        ERC721Metadata metadata;
    }
}

#[external]
#[inherit(ERC721, ERC721Metadata)]
impl Token {
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        base_uri: String,
    ) {
        self.metadata.constructor(name, symbol, base_uri);
    }
}
