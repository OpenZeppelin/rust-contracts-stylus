#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use alloy_primitives::{Address, U256};
use contracts::{
    erc721::{
        extensions::{ERC721Metadata, ERC721UriStorage},
        ERC721,
    },
    utils::Pausable,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, external, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC721 erc721;
        #[borrow]
        ERC721Metadata metadata;
        #[borrow]
        ERC721UriStorage uri_storage;
        #[borrow]
        Pausable pausable;
    }
}

#[external]
#[inherit(ERC721, ERC721Metadata, ERC721UriStorage, Pausable)]
impl Token {
    // We need to properly initialize all of Token's attributes.
    // For that, we need to call each attribute's constructor if it exists.
    //
    // NOTE: This is a temporary solution for state initialization.
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        base_uri: String,
    ) {
        self.metadata.constructor(name, symbol, base_uri);
    }

    // Overrides [`ERC721UriStorage::token_uri`].
    // Returns the Uniform Resource Identifier (URI) for tokenId token.
    pub fn token_uri(&self, token_id: U256) -> String {
        let base = self.metadata.base_uri();
        let token_uri = self.uri_storage.token_uri(token_id);

        // If there is no base URI, return the token URI.
        if base.is_empty() {
            return token_uri;
        }

        let mut uri = base;
        // If both are set, concatenate the baseURI and tokenURI (via
        // string.concat).
        if !token_uri.is_empty() {
            uri.push_str(&token_uri);
        } else {
            uri.push_str(&token_id.to_string());
        }

        uri
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc721.safe_transfer_from(from, to, token_id).map_err(|e| e.into())
    }

    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc721
            .safe_transfer_from_with_data(from, to, token_id, data)
            .map_err(|e| e.into())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc721.transfer_from(from, to, token_id).map_err(|e| e.into())
    }
}
