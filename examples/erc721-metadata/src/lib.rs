#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{
    extensions::{
        Erc721Metadata as Metadata, Erc721UriStorage as UriStorage,
        IErc721Burnable, IErc721Metadata,
    },
    Erc721, IErc721,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc721MetadataExample {
        #[borrow]
        Erc721 erc721;
        #[borrow]
        Metadata metadata;
        #[borrow]
        UriStorage uri_storage;
    }
}

#[external]
#[inherit(Erc721, Metadata, UriStorage)]
impl Erc721MetadataExample {
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc721._mint(to, token_id)?)
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc721.burn(token_id)?)
    }

    // Overrides [`Erc721UriStorage::token_uri`].
    // Returns the Uniform Resource Identifier (URI) for tokenId token.
    #[selector(name = "tokenURI")]
    pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
        let _owner = self.erc721.owner_of(token_id)?;

        let base = self.metadata.base_uri();
        let token_uri = self.uri_storage.token_uri(token_id);

        // If there is no base URI, return the token URI.
        if base.is_empty() {
            return Ok(token_uri);
        }

        // If both are set,
        // concatenate the base URI and token URI.
        let uri = if !token_uri.is_empty() {
            base + &token_uri
        } else {
            base + &token_id.to_string()
        };

        Ok(uri)
    }

    #[selector(name = "setTokenURI")]
    pub fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage._set_token_uri(token_id, token_uri)
    }
}
