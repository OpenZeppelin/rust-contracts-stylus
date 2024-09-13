//! ERC-1155 token with storage based token URI management.
//!
//! Inspired by the [contracts::token::erc721::extensions::Erc721UriStorage]
use alloc::string::String;

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage};
use stylus_sdk::evm;

sol! {

    /// Emitted when the URI for token type `token_id` changes to `value`, if it is a non-programmatic URI.
    ///
    /// If an [`URI`] event was emitted for `token_id`, the standard
    /// https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions[guarantees] that `value` will equal the value
    /// returned by {IERC1155MetadataURI-uri}.
    #[allow(missing_docs)]
    event URI(string value, uint256 indexed token_id);
}

sol_storage! {
    /// Uri Storage.
    pub struct Erc1155UriStorage {
        /// Optional mapping for token URIs.
        mapping(uint256 => string) _token_uris;
        /// Optional base URI
        string _base_uri;
    }
}

impl Erc1155UriStorage {
    /// Sets `token_uri` as the `_token_uris` of `token_id`.
    pub fn _set_uri(&mut self, token_id: U256, token_uri: String) {
        self._token_uris.setter(token_id).set_str(token_uri);
        evm::log(URI { value: self.uri(token_id), token_id });
    }

    /// Sets `base_uri` as the `_base_uri` for all tokens
    pub fn _set_base_uri(&mut self, base_uri: String) {
        self._base_uri.set_str(base_uri);
    }
}

#[external]
impl Erc1155UriStorage {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    #[must_use]
    pub fn uri(&self, token_id: U256) -> String {
        if !self._token_uris.getter(token_id).is_empty() {
            let update_uri = self._base_uri.get_string()
                + &self._token_uris.getter(token_id).get_string();
            update_uri
        } else {
            todo!()
        }
    }
}
