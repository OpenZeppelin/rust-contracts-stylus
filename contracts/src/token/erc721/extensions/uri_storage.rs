//! ERC-721 token with storage-based token URI management.
//!
//! It also implements IERC4096, which is an ERC-721 Metadata Update Extension.
use alloc::string::{String, ToString};
use core::marker::PhantomData;

use alloy_primitives::U256;
use alloy_sol_types::sol;
use openzeppelin_stylus_proc::r#override;
use stylus_proc::{external, sol_storage};
use stylus_sdk::{evm, prelude::TopLevelStorage};

use crate::token::erc721::{
    extensions::metadata::{Erc721Metadata, IErc721Metadata},
    Erc721, Error, IErc721, IErc721Virtual,
};

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
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Erc721UriStorage<This: IErc721Virtual> {
        /// Optional mapping for token URIs.
        mapping(uint256 => string) _token_uris;
        PhantomData<This> _phantom_data;
    }
}

impl<This: IErc721Virtual> Erc721UriStorage<This> {
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
}

#[external]
impl<This: IErc721Virtual> Erc721UriStorage<This> {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    #[selector(name = "tokenURI")]
    pub fn token_uri(
        storage: &mut impl TopLevelStorage,
        token_id: U256,
    ) -> Result<String, Error> {
        let _owner = storage.inner::<Erc721<This>>().owner_of(token_id)?;

        let base = storage.inner::<Erc721Metadata>().base_uri();
        let token_uri = storage.inner::<Self>()._token_uri(token_id);

        // If there is no base URI, return the token URI.
        if base.is_empty() {
            return Ok(token_uri);
        }

        // If both are set,
        // concatenate the base URI and token URI.
        let uri = if !token_uri.is_empty() {
            base + &token_uri
        } else {
            base + &token_id.to_string()
        };

        Ok(uri)
    }

    #[selector(name = "setTokenURI")]
    pub fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
        self._set_token_uri(token_id, token_uri)
    }
}

#[r#override]
impl IErc721Virtual for Erc721UriStorageOverride {}

impl<This: IErc721Virtual> Erc721UriStorage<This> {
    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    #[must_use]
    pub fn _token_uri(&self, token_id: U256) -> String {
        self._token_uris.getter(token_id).get_string()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;

    use super::Erc721UriStorage;
    use crate::token::erc721::Erc721Override;

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        U256::from(num)
    }

    type Override = Erc721Override;

    #[motsu::test]
    fn get_token_uri_works(contract: Erc721UriStorage<Override>) {
        let token_id = random_token_id();

        let token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract._token_uris.setter(token_id).set_str(token_uri.clone());

        assert_eq!(token_uri, contract._token_uri(token_id));
    }

    #[motsu::test]
    fn set_token_uri_works(contract: Erc721UriStorage<Override>) {
        let token_id = random_token_id();

        let initial_token_uri = String::from("https://docs.openzeppelin.com/contracts/5.x/api/token/erc721#Erc721URIStorage");
        contract._token_uris.setter(token_id).set_str(initial_token_uri);

        let token_uri = String::from("Updated Token URI");
        contract._set_token_uri(token_id, token_uri.clone());

        assert_eq!(token_uri, contract._token_uri(token_id));
    }
}
