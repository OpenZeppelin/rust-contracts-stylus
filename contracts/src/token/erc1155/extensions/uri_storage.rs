//! ERC-1155 token with storage based token URI management.
//!
//! Inspired by the [`crate::token::erc721::extensions::Erc721UriStorage`]
use alloc::string::String;

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_proc::{public, sol_storage};
use stylus_sdk::evm;

sol! {

    /// Emitted when the URI for token type `token_id` changes to `value`, if it is a non-programmatic URI.
    ///
    /// If an [`URI`] event was emitted for `token_id`, the standard
    /// https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions[guarantees] that `value` will equal the value
    /// returned by [`Self::uri`].
    #[allow(missing_docs)]
    event URI(string value, uint256 indexed token_id);
}

sol_storage! {
    /// Uri Storage.
    pub struct Erc1155UriStorage {
        /// Optional mapping for token URIs.
        mapping(uint256 => string) _token_uris;
    }
}

impl Erc1155UriStorage {
    /// Sets `token_uri` as the `_token_uris` of `token_id`.
    pub fn _set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self._token_uris.setter(token_id).set_str(token_uri);
        evm::log(URI { value: self.uri(token_id), token_id });
    }
}

#[public]
impl Erc1155UriStorage {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    #[must_use]
    pub fn uri(&self, token_id: U256) -> String {
        self._token_uris.getter(token_id).get_string()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::contract;

    use super::Erc1155UriStorage;

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        U256::from(num)
    }

    #[motsu::test]
    fn test_get_uri(contract: Erc1155UriStorage) {
        let token_id = random_token_id();

        let token_uri = "https://docs.openzeppelin.com/contracts/5.x/api/token/erc1155#ERC1155URIStorage".to_string();

        contract._token_uris.setter(token_id).set_str(token_uri.clone());

        assert_eq!(token_uri, contract.uri(token_id));
    }

    #[motsu::test]
    fn test_set_uri(contract: Erc1155UriStorage) {
        let token_id = random_token_id();

        let token_uri = "https://docs.openzeppelin.com/contracts/5.x/api/token/erc1155#ERC1155URIStorage".to_string();

        contract._set_token_uri(token_id, token_uri.clone());

        assert_eq!(token_uri, contract.uri(token_id));
    }
}
