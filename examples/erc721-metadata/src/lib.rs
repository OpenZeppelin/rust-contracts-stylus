#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{
    extensions::{
        Erc721Metadata as Metadata, Erc721UriStorage as UriStorage,
        IErc721Burnable,
    },
    Erc721,
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc721MetadataExample {
        #[borrow]
        Erc721 erc721;
        #[borrow]
        Metadata metadata;
        UriStorage uri_storage;
    }
}

#[public]
#[inherit(Erc721, Metadata)]
impl Erc721MetadataExample {
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc721._mint(to, token_id)?)
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc721.burn(token_id)?)
    }

    #[selector(name = "tokenURI")]
    pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
        Ok(self.uri_storage.token_uri(
            token_id,
            &self.erc721,
            &self.metadata,
        )?)
    }

    #[selector(name = "setTokenURI")]
    pub fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage._set_token_uri(token_id, token_uri)
    }
}
