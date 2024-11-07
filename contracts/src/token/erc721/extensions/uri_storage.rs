//! ERC-721 token with storage-based token URI management.
//!
//! It also implements IERC4096, which is an ERC-721 Metadata Update Extension.
use alloc::string::String;

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_sdk::{evm, stylus_proc::sol_storage};

use crate::token::erc721::{extensions::Erc721Metadata, Error, IErc721};

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

sol_storage! {
    /// Uri Storage.
    pub struct Erc721UriStorage {
        /// Optional mapping for token URIs.
        mapping(uint256 => string) _token_uris;
    }
}

impl Erc721UriStorage {
    /// Sets `token_uri` as the tokenURI of `token_id`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `token_uri` - URI for the token.
    ///
    /// # Events
    /// Emits a [`MetadataUpdate`] event.
    pub fn _set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self._token_uris.setter(token_id).set_str(token_uri);
        evm::log(MetadataUpdate { token_id });
    }

    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
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
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    ///
    /// NOTE: In order to have [`Erc721UriStorage::token_uri`] exposed in ABI,
    /// you need to do this manually.
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
        let uri = if !token_uri.is_empty() {
            base + &token_uri
        } else {
            metadata.token_uri(token_id, erc721)?
        };

        Ok(uri)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::{msg, stylus_proc::sol_storage};

    use super::Erc721UriStorage;
    use crate::token::erc721::{extensions::Erc721Metadata, Erc721};

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        U256::from(num)
    }

    sol_storage! {
        struct Erc721MetadataExample {
            Erc721 erc721;
            Erc721Metadata metadata;
            Erc721UriStorage uri_storage;
        }
    }

    #[motsu::test]
    fn get_token_uri_works(contract: Erc721MetadataExample) {
        let alice = msg::sender();

        let token_id = random_token_id();

        contract
            .erc721
            ._mint(alice, token_id)
            .expect("should mint a token for Alice");

        let token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract
            .uri_storage
            ._token_uris
            .setter(token_id)
            .set_str(token_uri.clone());

        assert_eq!(
            token_uri,
            contract
                .uri_storage
                .token_uri(token_id, &contract.erc721, &contract.metadata)
                .expect("should return token URI")
        );
    }

    #[motsu::test]
    fn set_token_uri_works(contract: Erc721MetadataExample) {
        let alice = msg::sender();

        let token_id = random_token_id();

        contract
            .erc721
            ._mint(alice, token_id)
            .expect("should mint a token for Alice");

        let initial_token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract
            .uri_storage
            ._token_uris
            .setter(token_id)
            .set_str(initial_token_uri);

        let token_uri = String::from("Updated Token URI");
        contract.uri_storage._set_token_uri(token_id, token_uri.clone());

        assert_eq!(
            token_uri,
            contract
                .uri_storage
                .token_uri(token_id, &contract.erc721, &contract.metadata)
                .expect("should return token URI")
        );
    }
}
