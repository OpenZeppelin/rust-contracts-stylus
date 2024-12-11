//! ERC-1155 token with storage based token URI management.
//!
//! Inspired by the [`crate::token::erc721::extensions::Erc721UriStorage`]
use alloc::string::String;

use alloy_primitives::U256;
use stylus_sdk::{evm, stylus_proc::sol_storage};

use super::metadata_uri::{IErc1155MetadataUri, URI};

sol_storage! {
    /// Uri Storage.
    pub struct Erc1155UriStorage {
        /// Optional base URI
        string _base_uri;
        /// Optional mapping for token URIs.
        mapping(uint256 => string) _token_uris;
    }
}

impl Erc1155UriStorage {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `metadata_uri` - Read access to a contract providing
    ///   [`IErc1155MetadataUri`] interface.
    ///
    /// NOTE: In order to have [`Erc1155UriStorage::uri`] exposed in ABI,
    /// you need to do this manually.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    ///     pub fn uri(&self, token_id: U256) -> String {
    ///         self.uri_storage.uri(token_id, &self.metadata_uri)
    ///     }
    pub fn uri(
        &self,
        token_id: U256,
        metadata_uri: &impl IErc1155MetadataUri,
    ) -> String {
        let token_uri = self._token_uris.get(token_id).get_string();

        if token_uri.is_empty() {
            metadata_uri.uri(token_id)
        } else {
            self._base_uri.get_string() + &token_uri
        }
    }

    /// Sets `token_uri` as the tokenURI of `token_id`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `token_uri` - URI for the token.
    /// * `metadata_uri` - Read access to a contract providing
    ///   [`IErc1155MetadataUri`] interface.
    ///
    /// # Events
    ///
    /// Emits a [`URI`] event.
    pub fn set_token_uri(
        &mut self,
        token_id: U256,
        token_uri: String,
        metadata_uri: &impl IErc1155MetadataUri,
    ) {
        self._token_uris.setter(token_id).set_str(token_uri);
        evm::log(URI { value: self.uri(token_id, metadata_uri), id: token_id });
    }

    /// Sets `base_uri` as the `_base_uri` for all tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `base_uri` - New base URI.
    pub fn set_base_uri(&mut self, base_uri: String) {
        self._base_uri.set_str(base_uri);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    /*use alloy_primitives::U256;
    use stylus_sdk::stylus_proc::sol_storage;

    use super::Erc1155UriStorage;
    use crate::token::erc1155::{extensions::Erc1155MetadataUri, Erc1155};

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        U256::from(num)
    }

    sol_storage! {
        struct Erc1155Example {
            Erc1155 erc1155;
            Erc1155MetadataUri metadata_uri;
            Erc1155UriStorage uri_storage;
        }
    }

    #[motsu::test]
    fn uri_returns_metadata_uri_when_token_uri_is_not_set(
        contract: Erc1155Example,
    ) {
        let token_id = random_token_id();
        let uri = "https://some.metadata/token/uri";

        contract.metadata_uri._uri.set_str(uri.to_owned());

        assert_eq!(
            uri,
            contract.uri_storage.uri(token_id, &contract.metadata_uri)
        );
    }

    #[motsu::test]
    fn uri_returns_empty_string_when_no_uri_is_set(contract: Erc1155Example) {
        let token_id = random_token_id();

        assert!(contract
            .uri_storage
            .uri(token_id, &contract.metadata_uri)
            .is_empty());
    }

    #[motsu::test]
    fn uri_returns_token_uri_when_base_uri_is_empty(contract: Erc1155Example) {
        let token_id = random_token_id();
        let token_uri = "https://some.short/token/uri";

        contract
            .uri_storage
            ._token_uris
            .setter(token_id)
            .set_str(token_uri.to_owned());

        assert_eq!(
            token_uri,
            contract.uri_storage.uri(token_id, &contract.metadata_uri)
        );
    }

    #[motsu::test]
    fn uri_returns_concatenated_base_uri_and_token_uri(
        contract: Erc1155Example,
    ) {
        let token_id = random_token_id();
        let base_uri = "https://some.base.uri";
        let token_uri = "/some/token/uri";

        contract.uri_storage._base_uri.set_str(base_uri.to_owned());
        contract
            .uri_storage
            ._token_uris
            .setter(token_id)
            .set_str(token_uri.to_owned());

        assert_eq!(
            base_uri.to_string() + token_uri,
            contract.uri_storage.uri(token_id, &contract.metadata_uri)
        );
    }

    #[motsu::test]
    fn uri_ignores_metadata_uri_when_token_uri_is_set(
        contract: Erc1155Example,
    ) {
        let token_id = random_token_id();
        let uri = "https://some.metadata/token/uri";
        let token_uri = "https://some.short/token/uri";

        contract.metadata_uri._uri.set_str(uri.to_owned());
        contract
            .uri_storage
            ._token_uris
            .setter(token_id)
            .set_str(token_uri.to_owned());

        assert_eq!(
            token_uri,
            contract.uri_storage.uri(token_id, &contract.metadata_uri)
        );
    }

    #[motsu::test]
    fn test_set_uri(contract: Erc1155Example) {
        let token_id = random_token_id();
        let uri = "https://some.metadata/token/uri";
        let token_uri = "https://some.short/token/uri".to_string();

        contract.metadata_uri._uri.set_str(uri.to_owned());

        contract.uri_storage.set_token_uri(
            token_id,
            token_uri.clone(),
            &contract.metadata_uri,
        );

        assert_eq!(
            token_uri,
            contract.uri_storage.uri(token_id, &contract.metadata_uri)
        );
    }

    #[motsu::test]
    fn test_set_base_uri(contract: Erc1155UriStorage) {
        let base_uri = "https://docs.openzeppelin.com/".to_string();
        contract.set_base_uri(base_uri.clone());

        assert_eq!(base_uri, contract._base_uri.get_string());
    }*/
}
