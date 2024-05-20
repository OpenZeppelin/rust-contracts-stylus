#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::U256;
use contracts::erc721::{
    extensions::{ERC721Metadata, IERC721Burnable},
    ERC721,
};
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
    // We need to properly initialize all Token's attributes.
    // For that we need to call each attributes' constructor if exists.
    //
    // NOTE: This is a temporary solution for state initialization.
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        base_uri: String,
    ) {
        self.metadata.constructor(name, symbol, base_uri);
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721.burn(token_id).map_err(|e| e.into())
    }
}
