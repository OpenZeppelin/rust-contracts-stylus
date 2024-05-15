//! ERC-721 token with storage-based token URI management.
//!
//! It also implements IERC4096, which is an ERC-721 Metadata Update Extension.
use alloc::string::String;

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage};
use stylus_sdk::evm;

sol_storage! {
    /// Uri Storage.
    pub struct ERC721UriStorage {
        /// Optional mapping for token URIs.
        mapping(uint256 => string) _token_uris;
    }
}

sol! {
    /// This event gets emitted when the metadata of a token is changed.
    ///
    /// The event comes from IERC4096.
    #[allow(missing_docs)]
    event MetadataUpdate(uint256 token_id);

    /// This event gets emitted when the metadata of a range of tokens is changed.
    ///
    /// Event comes from IERC4096.
    #[allow(missing_docs)]
    event BatchMetadataUpdate(uint256 from_token_id, uint256 to_token_id);
}

#[external]
impl ERC721UriStorage {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    #[must_use]
    pub fn token_uri(&self, token_id: U256) -> String {
        self._token_uris.get(token_id).get_string()
    }

    /// Sets `token_uri` as the tokenURI of `token_id`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `token_uri` - .
    ///
    /// # Events
    /// May emit a [`MetadataUpdate`] event.
    pub fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self._token_uris.setter(token_id).set_str(token_uri);
        evm::log(MetadataUpdate { token_id });
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::{prelude::StorageType, storage::StorageMap};

    use super::ERC721UriStorage;

    impl Default for ERC721UriStorage {
        fn default() -> Self {
            let root = U256::ZERO;

            ERC721UriStorage {
                _token_uris: unsafe { StorageMap::new(root, 0) },
            }
        }
    }

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        num.try_into().expect("conversion to U256")
    }

    #[grip::test]
    fn get_token_uri_works(contract: ERC721UriStorage) {
        let token_id = random_token_id();

        let token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#ERC721URIStorage");
        contract._token_uris.setter(token_id).set_str(token_uri.clone());

        assert_eq!(token_uri, contract.token_uri(token_id));
    }

    #[grip::test]
    fn set_token_uri_works(contract: ERC721UriStorage) {
        let token_id = random_token_id();

        let initial_token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#ERC721URIStorage");
        contract._token_uris.setter(token_id).set_str(initial_token_uri);

        let token_uri = String::from("Updated Token URI");
        contract.set_token_uri(token_id, token_uri.clone());

        assert_eq!(token_uri, contract.token_uri(token_id));
    }
}
