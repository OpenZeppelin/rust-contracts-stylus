#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{
    extensions::{
        Erc721Metadata as Metadata, Erc721URIStorage as URIStorage,
        IErc721Metadata,
    },
    Erc721,
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
        URIStorage uri_storage;
    }
}

#[external]
#[inherit(Erc721, Metadata, URIStorage)]
impl Erc721MetadataExample {
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        self.erc721._mint(to, token_id)?;

        Ok(())
    }

    // Overrides [`Erc721UriStorage::token_uri`].
    // Returns the Uniform Resource Identifier (URI) for tokenId token.
    pub fn token_uri(&self, token_id: U256) -> String {
        let base = self.metadata.base_uri();
        let token_uri = self.uri_storage.token_uri(token_id);

        // If there is no base URI, return the token URI.
        if base.is_empty() {
            return token_uri;
        }

        // If both are set,
        // concatenate the base URI and token URI.
        if !token_uri.is_empty() {
            base + &token_uri
        } else {
            base + &token_id.to_string()
        }
    }

    pub fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage._set_token_uri(token_id, token_uri)
    }
}
