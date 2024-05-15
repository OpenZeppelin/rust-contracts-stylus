//! ERC-721 token with storage based token URI management.
//! https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#ERC721URIStorage
//!
//! It also implements interface of the ERC-165 standard, as defined in the
//! https://eips.ethereum.org/EIPS/eip-165[ERC].
//! Implementers can declare support of contract interfaces, which can then be
//! queried by others ({ERC165Checker}).
//!
//! It also implements IERC4096 that is an ERC-721 Metadata Update Extension.
use alloc::string::String;

use alloy_primitives::{fixed_bytes, FixedBytes, U256};
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage};
use stylus_sdk::evm;

sol_storage! {
    /// Uri Storage.
    pub struct ERC721UriStorage {
        /// Optional mapping for token URIs
        mapping(uint256 => string) _token_uris;
    }
}

sol! {
    /// This event emits when the metadata of a token is changed.
    /// So that the third-party platforms such as NFT market could
    /// timely update the images and related attributes of the NFT.
    ///
    /// Event comes from IERC409.
    event MetadataUpdate(uint256 token_id);

    /// This event emits when the metadata of a range of tokens is changed.
    /// So that the third-party platforms such as NFT market could
    /// timely update the images and related attributes of the NFTs.
    ///
    /// Event comes from IERC409.
    event BatchMetadataUpdate(uint256 from_token_id, uint256 to_token_id);
}

#[external]
impl ERC721UriStorage {
    /// Returns true if this contract implements the interface defined by
    /// `interface_id`. See the corresponding
    /// https://eips.ethereum.org/EIPS/eip-165#how-interfaces-are-identified[ERC section]
    /// to learn more about how these ids are created.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `interface_id` - Interface id to be checked.
    pub fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        // Interface ID as defined in ERC-4906. This does not correspond to a
        // traditional interface ID as ERC-4906 only defines events and does not
        // include any external function.
        fixed_bytes!("49064906") == interface_id
    }

    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    pub fn token_uri(&self, token_id: U256) -> String {
        //
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
    use alloy_primitives::{fixed_bytes, U256};
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
    fn supports_interface_works(contract: ERC721UriStorage) {
        let proper_interface = fixed_bytes!("49064906");
        let improper_interface = fixed_bytes!("06494906");

        assert_eq!(true, contract.supports_interface(proper_interface));
        assert_eq!(false, contract.supports_interface(improper_interface));
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
