#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc1155::{
        extensions::{Erc1155MetadataUri, Erc1155UriStorage},
        Erc1155,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc1155MetadataUriExample {
        #[borrow]
        Erc1155 erc1155;
        #[borrow]
        Erc1155MetadataUri metadata_uri;
        Erc1155UriStorage uri_storage;
    }
}

#[public]
#[inherit(Erc1155, Erc1155MetadataUri)]
impl Erc1155MetadataUriExample {
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

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc1155::supports_interface(interface_id)
            || Erc1155MetadataUri::supports_interface(interface_id)
    }
}
