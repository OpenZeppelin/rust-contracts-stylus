#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        self,
        extensions::{
            Erc721Metadata as Metadata, Erc721UriStorage as UriStorage,
            IErc721Burnable,
        },
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
    metadata: Metadata,
    uri_storage: UriStorage,
}

#[public]
#[inherit(Erc721, Metadata)]
impl Erc721MetadataExample {
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
            || Metadata::supports_interface(interface_id)
    }
}
