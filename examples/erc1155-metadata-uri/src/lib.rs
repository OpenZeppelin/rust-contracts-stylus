#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::U256;
use openzeppelin_stylus::token::erc1155::{
    extensions::{Erc1155MetadataUri, Erc1155UriStorage},
    // Erc1155,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc1155MetadataUriExample {
    // TODO: contract size becomes too large, so uncomment this when the SDK
    // produces a smaller contract.
    // #[borrow]
    // erc1155: Erc1155,
    metadata_uri: Erc1155MetadataUri,
    uri_storage: Erc1155UriStorage,
}

#[public]
// TODO: contract size becomes too large, so uncomment this when the SDK
// produces a smaller contract.
// #[inherit(Erc1155)]
impl Erc1155MetadataUriExample {
    #[constructor]
    fn constructor(&mut self, uri: String) {
        self.metadata_uri.constructor(uri);
    }

    fn uri(&self, token_id: U256) -> String {
        self.uri_storage.uri(token_id, &self.metadata_uri)
    }

    #[selector(name = "setTokenURI")]
    fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage.set_token_uri(token_id, token_uri, &self.metadata_uri)
    }

    #[selector(name = "setBaseURI")]
    fn set_base_uri(&mut self, base_uri: String) {
        self.uri_storage.set_base_uri(base_uri)
    }
}
