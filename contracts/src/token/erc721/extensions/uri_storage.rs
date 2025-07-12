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
    #[allow(clippy::missing_errors_doc)]
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address};
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::*;
    use crate::{
        token::erc721::{self, extensions::Erc721Metadata, Erc721},
        utils::introspection::erc165::IErc165,
    };
    const TOKEN_ID: U256 = uint!(1_U256);
    use alloy_primitives::FixedBytes;

    #[storage]
    struct Erc721MetadataExample {
        pub erc721: Erc721,
        pub metadata: Erc721Metadata,
        pub uri_storage: Erc721UriStorage,
    }

    #[public]
    #[implements(IErc721Metadata<Error = erc721::Error>, IErc165)]
    impl Erc721MetadataExample {
        #[constructor]
        fn constructor(&mut self, name: String, symbol: String) {
            self.metadata.constructor(name, symbol);
        }

        #[selector(name = "setTokenURI")]
        fn set_token_uri(&mut self, token_id: U256, token_uri: String) {
            self.uri_storage._set_token_uri(token_id, token_uri);
        }
    }

    #[public]
    impl IErc721Metadata for Erc721MetadataExample {
        type Error = erc721::Error;

        fn name(&self) -> String {
            self.metadata.name()
        }

        fn symbol(&self) -> String {
            self.metadata.symbol()
        }

        #[selector(name = "tokenURI")]
        fn token_uri(&self, token_id: U256) -> Result<String, erc721::Error> {
            self.uri_storage.token_uri(token_id, &self.erc721, &self.metadata)
        }
    }

    #[public]
    impl IErc165 for Erc721MetadataExample {
        fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
            <Self as IErc721Metadata>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    unsafe impl TopLevelStorage for Erc721MetadataExample {}

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721MetadataExample as IErc721Metadata>::interface_id();
        let expected: FixedBytes<4> = 0x5b5e139f.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(
        contract: Contract<Erc721MetadataExample>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).supports_interface(
            <Erc721MetadataExample as IErc721Metadata>::interface_id()
        ));
        assert!(contract.sender(alice).supports_interface(
            <Erc721MetadataExample as IErc165>::interface_id()
        ));

        let fake_interface_id: FixedBytes<4> = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
    #[motsu::test]
    fn token_uri_returns_configured_metadata(
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
