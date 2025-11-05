//! ERC-1155 token with storage based token URI management.
//!
//! Inspired by the [`crate::token::erc721::extensions::Erc721UriStorage`]
use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::U256;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    evm,
    prelude::*,
    storage::{StorageMap, StorageString},
};

use super::metadata_uri::{IErc1155MetadataUri, URI};

/// State of an [`Erc1155UriStorage`] contract.
#[storage]
pub struct Erc1155UriStorage {
    /// Optional base URI.
    pub(crate) base_uri: StorageString,
    /// Optional mapping for token URIs.
    pub(crate) token_uris: StorageMap<U256, StorageString>,
}

/// Interface of an optional extension for ERC-1155 with storage based token URI
/// management.
#[interface_id]
pub trait IErc1155UriStorage {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    fn uri(&self, token_id: U256) -> String;
}

impl Erc1155UriStorage {
    /// See [`IErc1155UriStorage::uri`].
    pub fn uri(
        &self,
        token_id: U256,
        metadata_uri: &impl IErc1155MetadataUri,
    ) -> String {
        let token_uri = self.token_uris.get(token_id).get_string();

        if token_uri.is_empty() {
            metadata_uri.uri(token_id)
        } else {
            self.base_uri.get_string() + &token_uri
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
    /// * [`URI`].
    pub fn set_token_uri(
        &mut self,
        token_id: U256,
        token_uri: String,
        metadata_uri: &impl IErc1155MetadataUri,
    ) {
        self.token_uris.setter(token_id).set_str(token_uri);
        evm::log(URI { value: self.uri(token_id, metadata_uri), id: token_id });
    }

    /// Sets `base_uri` as the `base_uri` for all tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `base_uri` - New base URI.
    pub fn set_base_uri(&mut self, base_uri: String) {
        self.base_uri.set_str(base_uri);
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::*;

    use super::*;
    use crate::token::erc1155::extensions::Erc1155MetadataUri;

    #[storage]
    struct Erc1155MetadataExample {
        pub metadata_uri: Erc1155MetadataUri,
        pub uri_storage: Erc1155UriStorage,
    }

    #[public]
    #[implements(IErc1155UriStorage)]
    impl Erc1155MetadataExample {}

    #[public]
    impl IErc1155UriStorage for Erc1155MetadataExample {
        fn uri(&self, token_id: U256) -> String {
            self.uri_storage.uri(token_id, &self.metadata_uri)
        }
    }

    unsafe impl TopLevelStorage for Erc1155MetadataExample {}

    const TOKEN_ID: U256 = U256::ONE;

    #[motsu::test]
    fn uri_returns_metadata_uri_when_token_uri_is_not_set(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let uri = "https://some.metadata/token/uri";

        contract.sender(alice).metadata_uri.uri.set_str(uri);

        assert_eq!(uri, contract.sender(alice).uri(TOKEN_ID));
    }

    #[motsu::test]
    fn uri_returns_empty_string_when_no_uri_is_set(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).uri(TOKEN_ID).is_empty());
    }

    #[motsu::test]
    fn uri_returns_token_uri_when_base_uri_is_empty(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let token_uri = "https://some.short/token/uri";

        contract
            .sender(alice)
            .uri_storage
            .token_uris
            .setter(TOKEN_ID)
            .set_str(token_uri);

        assert_eq!(token_uri, contract.sender(alice).uri(TOKEN_ID));
    }

    #[motsu::test]
    fn uri_returns_concatenated_base_uri_and_token_uri(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let base_uri = "https://some.base.uri";
        let token_uri = "/some/token/uri";

        contract.sender(alice).uri_storage.base_uri.set_str(base_uri);
        contract
            .sender(alice)
            .uri_storage
            .token_uris
            .setter(TOKEN_ID)
            .set_str(token_uri);

        assert_eq!(
            base_uri.to_string() + token_uri,
            contract.sender(alice).uri(TOKEN_ID)
        );
    }

    #[motsu::test]
    fn uri_ignores_metadata_uri_when_token_uri_is_set(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let uri = "https://some.metadata/token/uri";
        let token_uri = "https://some.short/token/uri";

        contract.sender(alice).metadata_uri.uri.set_str(uri);
        contract
            .sender(alice)
            .uri_storage
            .token_uris
            .setter(TOKEN_ID)
            .set_str(token_uri);

        assert_eq!(token_uri, contract.sender(alice).uri(TOKEN_ID));
    }
    #[motsu::test]
    fn test_set_uri(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let uri = "https://some.metadata/token/uri";
        let token_uri = "https://some.short/token/uri".to_string();

        contract.sender(alice).metadata_uri.uri.set_str(uri);
        contract.sender(alice).uri_storage.set_token_uri(
            TOKEN_ID,
            token_uri.clone(),
            &contract.sender(alice).metadata_uri,
        );

        assert_eq!(token_uri, contract.sender(alice).uri(TOKEN_ID));
    }
    #[motsu::test]
    fn test_set_base_uri(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let base_uri = "https://docs.openzeppelin.com/".to_string();
        contract.sender(alice).uri_storage.set_base_uri(base_uri.clone());

        assert_eq!(
            base_uri,
            contract.sender(alice).uri_storage.base_uri.get_string()
        );
    }
}
