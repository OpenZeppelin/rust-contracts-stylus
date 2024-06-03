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
            Erc721Enumerable as Enumerable, Erc721Metadata as Metadata,
            Erc721UriStorage as UriStorage, IErc721Burnable,
        },
        Erc721, IErc721,
    },
    utils::Pausable,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, external, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Erc721Example {
        #[borrow]
        Erc721 erc721;
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
#[inherit(Erc721, Enumerable, Metadata, Pausable, UriStorage)]
impl Erc721Example {
    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc721.burn(token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_all_tokens_enumeration(token_id);

        Ok(())
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc721._mint(to, token_id)?;

        // Update the extension's state.
        self.enumerable._add_token_to_all_tokens_enumeration(token_id);

        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.safe_transfer_from(from, to, token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

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
        self.pausable.when_not_paused()?;

        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.safe_transfer_from_with_data(from, to, token_id, data)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.transfer_from(from, to, token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    // Overrides [`Erc721UriStorage::token_uri`].
    // Returns the Uniform Resource Identifier (URI) for tokenId token.
    pub fn token_uri(&self, token_id: U256) -> String {
        let base = self.metadata.base_uri();
        let token_uri = self.uri_storage.token_uri(token_id);

        // If there is no base URI, return the token URI.
        if base.is_empty() {
            return token_uri;
        }

        // If both are set,
        // concatenate the base URI and token URI.
        if !token_uri.is_empty() {
            base + &token_uri
        } else {
            base + &token_id.to_string()
        }
    }
}
