#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        extensions::{
            Erc721Metadata as Metadata, Erc721UriStorage as UriStorage,
            IErc721Burnable,
        },
        Erc721,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc721MetadataExample {
    #[borrow]
    pub erc721: Erc721,
    #[borrow]
    pub metadata: Metadata,
    pub uri_storage: UriStorage,
}

#[public]
#[inherit(Erc721, Metadata)]
impl Erc721MetadataExample {
    #[constructor]
    fn constructor(&mut self, name: String, symbol: String, base_uri: String) {
        self.metadata.constructor(name, symbol);
        self.metadata._base_uri.set_str(base_uri);
    }

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

    pub fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(interface_id)
            || Metadata::supports_interface(interface_id)
    }
}
