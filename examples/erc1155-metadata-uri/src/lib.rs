#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use openzeppelin_stylus::{
    token::{
        erc1155,
        erc1155::{
            extensions::{
                Erc1155MetadataUri, Erc1155UriStorage, IErc1155MetadataUri,
            },
            Erc1155, IErc1155,
        },
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{aliases::B32, Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc1155MetadataUriExample {
    erc1155: Erc1155,
    metadata_uri: Erc1155MetadataUri,
    uri_storage: Erc1155UriStorage,
}

#[public]
#[implements(IErc1155<Error = erc1155::Error>, IErc1155MetadataUri, IErc165)]
impl Erc1155MetadataUriExample {
    #[constructor]
    fn constructor(&mut self, uri: String) {
        self.metadata_uri.constructor(uri);
    }

    #[selector(name = "setTokenURI")]
    fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self.uri_storage.set_token_uri(token_id, token_uri, &self.metadata_uri);
    }

    #[selector(name = "setBaseURI")]
    fn set_base_uri(&mut self, base_uri: String) {
        self.uri_storage.set_base_uri(base_uri);
    }
}

#[public]
impl IErc1155 for Erc1155MetadataUriExample {
    type Error = erc1155::Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.erc1155.balance_of(account, id)
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, Self::Error> {
        self.erc1155.balance_of_batch(accounts, ids)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error> {
        self.erc1155.set_approval_for_all(operator, approved)
    }

    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool {
        self.erc1155.is_approved_for_all(account, operator)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.erc1155.safe_transfer_from(from, to, id, value, data)
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.erc1155.safe_batch_transfer_from(from, to, ids, values, data)
    }
}

#[public]
impl IErc1155MetadataUri for Erc1155MetadataUriExample {
    fn uri(&self, token_id: U256) -> String {
        self.uri_storage.uri(token_id, &self.metadata_uri)
    }
}

#[public]
impl IErc165 for Erc1155MetadataUriExample {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc1155.supports_interface(interface_id)
            || self.metadata_uri.supports_interface(interface_id)
    }
}
