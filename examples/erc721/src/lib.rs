#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use alloy_primitives::U256;
use contracts::erc721::{
    extensions::{ERC721Metadata, ERC721UriStorage},
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
        #[borrow]
        ERC721UriStorage uri_storage;
    }
}

#[external]
#[inherit(ERC721, ERC721Metadata, ERC721UriStorage)]
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

    // Override [`ERC721UriStorage::token_uri`].
    // Provide concatenation of Base URI from [`ERC721Metadata`]
    // and `token_uri` from [`ERC721UriStorage`]
    pub fn token_uri(&self, token_id: U256) -> String {
        let mut uri = self.metadata.base_uri();
        let token_uri = self.uri_storage.token_uri(token_id);

        // Concatenate the Base URI and Token URI
        uri.push_str(&token_uri);

        uri
    }
}
