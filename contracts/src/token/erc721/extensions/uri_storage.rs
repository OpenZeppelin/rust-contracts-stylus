//! ERC-721 token with storage-based token URI management.
//!
//! It also implements IERC4096, which is an ERC-721 Metadata Update Extension.
use alloc::string::String;

use alloy_primitives::U256;
pub use sol::*;
use stylus_sdk::{
    evm,
    prelude::storage,
    storage::{StorageMap, StorageString},
};

use crate::token::erc721::{extensions::Erc721Metadata, Error, IErc721};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// This event gets emitted when the metadata of a token is changed.
        ///
        /// The event comes from IERC4096.
        #[allow(missing_docs)]
        event MetadataUpdate(uint256 token_id);

        /// This event gets emitted when the metadata of a range of tokens
        /// is changed.
        ///
        /// The event comes from IERC4096.
        #[allow(missing_docs)]
        event BatchMetadataUpdate(uint256 from_token_id, uint256 to_token_id);
    }
}

/// State of an [`Erc721UriStorage`] contract.
#[storage]
pub struct Erc721UriStorage {
    /// Optional mapping for token URIs.
    #[allow(clippy::used_underscore_binding)]
    pub _token_uris: StorageMap<U256, StorageString>,
}

impl Erc721UriStorage {
    /// Sets `token_uri` as the token URI of `token_id`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `token_uri` - URI for the token.
    ///
    /// # Events
    ///
    /// * [`MetadataUpdate`].
    pub fn _set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self._token_uris.setter(token_id).set_str(token_uri);
        evm::log(MetadataUpdate { token_id });
    }

    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `token_id`
    /// parameter. Both the `erc721` and `metadata` references should come from
    /// your contract's state. The implementation should use
    /// `#[selector(name = "tokenURI")]` to match Solidity's camelCase naming
    /// convention and it should forward the call to your internal storage
    /// instance along with both references.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `erc721` - Read access to a contract providing [`IErc721`] interface.
    /// * `metadata` - Read access to a [`Erc721Metadata`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[selector(name = "tokenURI")]
    /// pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
    ///     Ok(self.uri_storage.token_uri(
    ///        token_id,
    ///        &self.erc721,
    ///        &self.metadata,
    ///    )?)
    /// }
    pub fn token_uri(
        &self,
        token_id: U256,
        erc721: &impl IErc721<Error = Error>,
        metadata: &Erc721Metadata,
    ) -> Result<String, Error> {
        let _owner = erc721.owner_of(token_id)?;

        let token_uri = self._token_uris.getter(token_id).get_string();
        let base = metadata.base_uri();

        // If there is no base URI, return the token URI.
        if base.is_empty() {
            return Ok(token_uri);
        }

        // If both are set, concatenate the `base_uri` and `token_uri`.
        let uri = if token_uri.is_empty() {
            metadata.token_uri(token_id, erc721)?
        } else {
            base + &token_uri
        };

        Ok(uri)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use stylus_sdk::{
        alloy_primitives::{uint, U256},
        msg,
        prelude::storage,
    };

    use super::Erc721UriStorage;
    use crate::token::erc721::{extensions::Erc721Metadata, Erc721};

    #[storage]
    struct Erc721MetadataExample {
        pub erc721: Erc721,
        pub metadata: Erc721Metadata,
        pub uri_storage: Erc721UriStorage,
    }

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn get_token_uri_works(contract: Erc721MetadataExample) {
        let alice = msg::sender();

        contract
            .erc721
            ._mint(alice, TOKEN_ID)
            .expect("should mint a token for Alice");

        let token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract
            .uri_storage
            ._token_uris
            .setter(TOKEN_ID)
            .set_str(token_uri.clone());

        assert_eq!(
            token_uri,
            contract
                .uri_storage
                .token_uri(TOKEN_ID, &contract.erc721, &contract.metadata)
                .expect("should return token URI")
        );
    }

    #[motsu::test]
    fn set_token_uri_works(contract: Erc721MetadataExample) {
        let alice = msg::sender();

        contract
            .erc721
            ._mint(alice, TOKEN_ID)
            .expect("should mint a token for Alice");

        let initial_token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract
            .uri_storage
            ._token_uris
            .setter(TOKEN_ID)
            .set_str(initial_token_uri);

        let token_uri = String::from("Updated Token URI");
        contract.uri_storage._set_token_uri(TOKEN_ID, token_uri.clone());

        assert_eq!(
            token_uri,
            contract
                .uri_storage
                .token_uri(TOKEN_ID, &contract.erc721, &contract.metadata)
                .expect("should return token URI")
        );
    }
}
