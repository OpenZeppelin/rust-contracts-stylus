//! ERC-721 token with storage-based token URI management.
//!
//! It also implements IERC4096, which is an ERC-721 Metadata Update Extension.
use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::U256;
pub use sol::*;
use stylus_sdk::{
    evm,
    prelude::*,
    storage::{StorageMap, StorageString},
};

use crate::token::erc721::{
    self,
    extensions::{Erc721Metadata, IErc721Metadata},
    IErc721,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// This event gets emitted when the metadata of a token is changed.
        ///
        /// The event comes from IERC4096.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event MetadataUpdate(uint256 token_id);

        /// This event gets emitted when the metadata of a range of tokens
        /// is changed.
        ///
        /// The event comes from IERC4096.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event BatchMetadataUpdate(uint256 from_token_id, uint256 to_token_id);
    }
}

/// State of an [`Erc721UriStorage`] contract.
#[storage]
pub struct Erc721UriStorage {
    /// Optional mapping for token URIs.
    pub token_uris: StorageMap<U256, StorageString>,
}

/// Interface of an optional extension ERC-721 token providing storage based
/// token URI management.
pub trait IErc721UriStorage: IErc721Metadata {}

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
        self.token_uris.setter(token_id).set_str(token_uri);
        evm::log(MetadataUpdate { token_id });
    }

    /// Check [`IErc721Metadata::token_uri()`] for more details.
    pub fn token_uri(
        &self,
        token_id: U256,
        erc721: &impl IErc721<Error = erc721::Error>,
        metadata: &Erc721Metadata,
    ) -> Result<String, erc721::Error> {
        erc721.owner_of(token_id)?;

        let token_uri = self.token_uris.getter(token_id).get_string();
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
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::Erc721UriStorage;
    use crate::token::erc721::{self, extensions::Erc721Metadata, Erc721};

    const TOKEN_ID: U256 = uint!(1_U256);

    #[storage]
    struct Erc721MetadataExample {
        pub erc721: Erc721,
        pub metadata: Erc721Metadata,
        pub uri_storage: Erc721UriStorage,
    }

    #[public]
    impl Erc721MetadataExample {
        #[selector(name = "tokenURI")]
        fn token_uri(&self, token_id: U256) -> Result<String, erc721::Error> {
            self.uri_storage.token_uri(token_id, &self.erc721, &self.metadata)
        }

        #[selector(name = "setTokenURI")]
        fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
            self.uri_storage._set_token_uri(token_id, token_uri);
        }
    }

    unsafe impl TopLevelStorage for Erc721MetadataExample {}

    #[motsu::test]
    fn token_uri_works(
        contract: Contract<Erc721MetadataExample>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            .erc721
            ._mint(alice, TOKEN_ID)
            .expect("should mint a token for Alice");

        let token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract.sender(alice).set_token_uri(TOKEN_ID, token_uri.clone());

        assert_eq!(
            token_uri,
            contract
                .sender(alice)
                .token_uri(TOKEN_ID)
                .expect("should return token URI")
        );
    }
}
