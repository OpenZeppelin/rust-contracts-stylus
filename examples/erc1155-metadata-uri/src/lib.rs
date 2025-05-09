#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::U256;
use openzeppelin_stylus::token::{
    erc1155,
    erc1155::extensions::{Erc1155MetadataUri, Erc1155UriStorage},
};
use stylus_sdk::prelude::*;

#[derive(SolidityError, Debug)]
enum Error {
    InsufficientBalance(erc1155::ERC1155InsufficientBalance),
    InvalidSender(erc1155::ERC1155InvalidSender),
    InvalidReceiver(erc1155::ERC1155InvalidReceiver),
    InvalidReceiverWithReason(erc1155::InvalidReceiverWithReason),
    MissingApprovalForAll(erc1155::ERC1155MissingApprovalForAll),
    InvalidApprover(erc1155::ERC1155InvalidApprover),
    InvalidOperator(erc1155::ERC1155InvalidOperator),
    InvalidArrayLength(erc1155::ERC1155InvalidArrayLength),
}

impl From<erc1155::Error> for Error {
    fn from(value: erc1155::Error) -> Self {
        match value {
            erc1155::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc1155::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc1155::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc1155::Error::InvalidReceiverWithReason(e) => {
                Error::InvalidReceiverWithReason(e)
            }
            erc1155::Error::MissingApprovalForAll(e) => {
                Error::MissingApprovalForAll(e)
            }
            erc1155::Error::InvalidApprover(e) => Error::InvalidApprover(e),
            erc1155::Error::InvalidOperator(e) => Error::InvalidOperator(e),
            erc1155::Error::InvalidArrayLength(e) => {
                Error::InvalidArrayLength(e)
            }
        }
    }
}

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
// #[implements(IErc1155<Error=Error>)]
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

// TODO: contract size becomes too large, so uncomment this when the SDK
// produces a smaller contract.
/*
#[public]
impl IErc1155 for Erc1155MetadataUriExample {
    type Error = Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.erc1155.balance_of(account, id)
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, <Self as IErc1155>::Error> {
        Ok(self.erc1155.balance_of_batch(accounts, ids)?)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), <Self as IErc1155>::Error> {
        Ok(self.erc1155.set_approval_for_all(operator, approved)?)
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
    ) -> Result<(), <Self as IErc1155>::Error> {
        Ok(self.erc1155.safe_transfer_from(from, to, id, value, data)?)
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), <Self as IErc1155>::Error> {
        Ok(self
            .erc1155
            .safe_batch_transfer_from(from, to, ids, values, data)?)
    }
}
*/
