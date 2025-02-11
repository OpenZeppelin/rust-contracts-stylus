//! ERC-1155 token with storage based token URI management.
//!
//! Inspired by the [`crate::token::erc721::extensions::Erc721UriStorage`]
use alloc::string::String;

use alloy_primitives::U256;
use stylus_sdk::{
    evm,
    prelude::storage,
    storage::{StorageMap, StorageString},
};

use super::metadata_uri::{IErc1155MetadataUri, URI};

/// State of an [`Erc1155UriStorage`] contract.
#[storage]
pub struct Erc1155UriStorage {
    /// Optional base URI.
    #[allow(clippy::used_underscore_binding)]
    pub _base_uri: StorageString,
    /// Optional mapping for token URIs.
    #[allow(clippy::used_underscore_binding)]
    pub _token_uris: StorageMap<U256, StorageString>,
}

impl Erc1155UriStorage {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `token_id`
    /// parameter. The `metadata_uri` reference should come from your contract's
    /// state. The implementation should forward the call to your internal
    /// storage instance along with the metadata reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `metadata_uri` - Read access to a contract providing
    ///   [`IErc1155MetadataUri`] interface.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    ///     fn uri(&self, token_id: U256) -> String {
    ///         self.uri_storage.uri(token_id, &self.metadata_uri)
    ///     }
    /// ```
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
    /// * [`URI`].
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
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::*;

    use super::Erc1155UriStorage;
    use crate::token::erc1155::extensions::Erc1155MetadataUri;

    #[storage]
    struct Erc1155MetadataExample {
        #[borrow]
        pub metadata_uri: Erc1155MetadataUri,
        pub uri_storage: Erc1155UriStorage,
    }

    #[public]
    impl Erc1155MetadataExample {
        fn uri(&self, token_id: U256) -> String {
            self.uri_storage.uri(token_id, &self.metadata_uri)
        }
    }

    unsafe impl TopLevelStorage for Erc1155MetadataExample {}

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn uri_returns_metadata_uri_when_token_uri_is_not_set(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let uri = "https://some.metadata/token/uri";

        contract.init(alice, |contract| {
            contract.metadata_uri._uri.set_str(uri.to_owned());
        });

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

        contract.init(alice, |contract| {
            contract
                .uri_storage
                ._token_uris
                .setter(TOKEN_ID)
                .set_str(token_uri.to_owned());
        });

        assert_eq!(token_uri, contract.sender(alice).uri(TOKEN_ID));
    }

    #[motsu::test]
    fn uri_returns_concatenated_base_uri_and_token_uri(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let base_uri = "https://some.base.uri";
        let token_uri = "/some/token/uri";

        contract.init(alice, |contract| {
            contract.uri_storage._base_uri.set_str(base_uri.to_owned());
            contract
                .uri_storage
                ._token_uris
                .setter(TOKEN_ID)
                .set_str(token_uri.to_owned());
        });

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

        contract.init(alice, |contract| {
            contract.metadata_uri._uri.set_str(uri.to_owned());
            contract
                .uri_storage
                ._token_uris
                .setter(TOKEN_ID)
                .set_str(token_uri.to_owned());
        });

        assert_eq!(token_uri, contract.sender(alice).uri(TOKEN_ID));
    }
    #[motsu::test]
    fn test_set_uri(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let uri = "https://some.metadata/token/uri";
        let token_uri = "https://some.short/token/uri".to_string();

        contract.init(alice, |contract| {
            contract.metadata_uri._uri.set_str(uri.to_owned());
            contract.uri_storage.set_token_uri(
                TOKEN_ID,
                token_uri.clone(),
                &contract.metadata_uri,
            );
        });

        assert_eq!(token_uri, contract.sender(alice).uri(TOKEN_ID));
    }
    #[motsu::test]
    fn test_set_base_uri(
        contract: Contract<Erc1155MetadataExample>,
        alice: Address,
    ) {
        let base_uri = "https://docs.openzeppelin.com/".to_string();
        contract.init(alice, |contract| {
            contract.uri_storage.set_base_uri(base_uri.clone());
        });

        assert_eq!(
            base_uri,
            contract.sender(alice).uri_storage._base_uri.get_string()
        );
    }
}
