#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use alloy_primitives::{Address, U256};
use contracts::{
    erc721::{
        extensions::{
            ERC721Enumerable as Enumerable, ERC721Metadata as Metadata,
            ERC721UriStorage as UriStorage, IERC721Burnable,
        },
        ERC721, IERC721,
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
        Enumerable enumerable;
        #[borrow]
        Metadata metadata;
        #[borrow]
        Pausable pausable;
        #[borrow]
        UriStorage uri_storage;
    }
}

#[external]
#[inherit(ERC721, Enumerable, Metadata, Pausable, UriStorage)]
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

    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        // Check `Unpaused` state.
        self.pausable.when_not_paused()?;

        // Perform [`ERC721`] action.
        self.erc721.burn(token_id)?;

        // Apply updates to Enumerable extension.
        self.enumerable._remove_token_from_all_tokens_enumeration(token_id);

        Ok(())
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        // Check `Unpaused` state.
        self.pausable.when_not_paused()?;

        // Perform [`ERC721`] action.
        self.erc721._mint(to, token_id)?;

        // Apply updates to Enumerable extension.
        self.enumerable._add_token_to_all_tokens_enumeration(token_id);

        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        // Check `Unpaused` state.
        self.pausable.when_not_paused()?;

        // Remember previous owner for Enumerable extension.
        let previous_owner = self.erc721.owner_of(token_id)?;

        // Perform [`ERC721`] action.
        self.erc721.safe_transfer_from(from, to, token_id)?;

        // Apply updates to Enumerable extension.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        );
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        );

        Ok(())
    }

    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        // Check `Unpaused` state.
        self.pausable.when_not_paused()?;

        // Remember previous owner for Enumerable extension.
        let previous_owner = self.erc721.owner_of(token_id)?;

        // Perform [`ERC721`] action.
        self.erc721.safe_transfer_from_with_data(from, to, token_id, data)?;

        // Apply updates to Enumerable extension.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        );
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        );

        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        // Check `Unpaused` state.
        self.pausable.when_not_paused()?;

        // Remember previous owner for Enumerable extension.
        let previous_owner = self.erc721.owner_of(token_id)?;

        // Perform [`ERC721`] action.
        self.erc721.transfer_from(from, to, token_id)?;

        // Apply updates to Enumerable extension.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        );
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        );

        Ok(())
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
}
