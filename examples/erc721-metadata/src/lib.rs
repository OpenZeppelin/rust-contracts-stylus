#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        self,
        extensions::{Erc721Metadata, Erc721UriStorage, IErc721Burnable},
        Erc721,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc721MetadataExample {
    #[borrow]
    erc721: Erc721,
    #[borrow]
    metadata: Erc721Metadata,
    uri_storage: Erc721UriStorage,
}

#[public]
#[inherit(Erc721, Erc721Metadata)]
impl Erc721MetadataExample {
    #[constructor]
    fn constructor(&mut self, name: String, symbol: String, base_uri: String) {
        self.metadata.constructor(name, symbol);
        self.metadata.base_uri.set_str(base_uri);
    }

    fn mint(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), erc721::Error> {
        self.erc721._mint(to, token_id)
    }

    fn burn(&mut self, token_id: U256) -> Result<(), erc721::Error> {
        self.erc721.burn(token_id)
    }

    #[selector(name = "tokenURI")]
    fn token_uri(&self, token_id: U256) -> Result<String, erc721::Error> {
        self.uri_storage.token_uri(token_id, &self.erc721, &self.metadata)
    }

    #[selector(name = "setTokenURI")]
    fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage._set_token_uri(token_id, token_uri)
    }

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(interface_id)
            || Erc721Metadata::supports_interface(interface_id)
    }
}
