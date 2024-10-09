#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc721::{
    extensions::{
        Erc721UriStorage as UriStorage, IErc721Burnable, IErc721Metadata,
    },
    Erc721,
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc721MetadataExample {
        #[borrow]
        Erc721 erc721;
        UriStorage uri_storage;
    }
}

#[public]
#[inherit(Erc721)]
impl Erc721MetadataExample {
    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc721._mint(to, token_id)?)
    }

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc721.burn(token_id)?)
    }

    fn name(&self) -> String {
        self.erc721.name()
    }

    fn symbol(&self) -> String {
        self.erc721.symbol()
    }

    /// Overrides [`Erc721::token_uri`].
    /// Returns the Uniform Resource Identifier (URI) for tokenId token.
    #[selector(name = "tokenURI")]
    pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
        Ok(self.uri_storage.token_uri(&self.erc721, token_id)?)
    }

    #[selector(name = "setTokenURI")]
    pub fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage._set_token_uri(token_id, token_uri)
    }
}
