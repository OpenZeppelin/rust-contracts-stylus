//! Optional Metadata of the ERC-721 standard.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    prelude::storage, storage::StorageString, stylus_proc::public,
};

use crate::{
    token::erc721::{Error, IErc721},
    utils::{introspection::erc165::IErc165, Metadata},
};

/// State of an [`Erc721Metadata`] contract.
#[storage]
pub struct Erc721Metadata {
    /// [`Metadata`] contract.
    #[allow(clippy::used_underscore_binding)]
    pub _metadata: Metadata,
    /// Base URI for tokens.
    #[allow(clippy::used_underscore_binding)]
    pub _base_uri: StorageString,
}

/// Interface for the optional metadata functions from the ERC-721 standard.
#[interface_id]
pub trait IErc721Metadata {
    /// Returns the token collection name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn name(&self) -> String;

    /// Returns token collection symbol.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn symbol(&self) -> String;
}

// FIXME: Apply multi-level inheritance to export Metadata's functions.
// With the current version of SDK it is not possible.
// See https://github.com/OffchainLabs/stylus-sdk-rs/pull/120
#[public]
impl IErc721Metadata for Erc721Metadata {
    fn name(&self) -> String {
        self._metadata.name()
    }

    fn symbol(&self) -> String {
        self._metadata.symbol()
    }
}

const TOKEN_URI_SELECTOR: u32 =
    u32::from_be_bytes(stylus_sdk::function_selector!("tokenURI", U256));

impl IErc165 for Erc721Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        // NOTE: interface id is calculated using additional selector
        //  [`Erc721Metadata::token_uri`]
        (<Self as IErc721Metadata>::INTERFACE_ID ^ TOKEN_URI_SELECTOR)
            == u32::from_be_bytes(*interface_id)
    }
}

impl Erc721Metadata {
    /// Returns the base of Uniform Resource Identifier (URI) for tokens'
    /// collection.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn base_uri(&self) -> String {
        self._base_uri.get_string()
    }

    /// Returns the Uniform Resource Identifier (URI) for `token_id` token.
    ///
    /// NOTE: To expose this function in your contract's ABI, implement it as
    /// shown in the Examples section below, accepting only the `token_id`
    /// parameter. The `erc721` reference should come from your contract's
    /// state. The implementation should use `#[selector(name = "tokenURI")]` to
    /// match Solidity's camelCase naming convention and it should forward the
    /// call to your internal storage instance along with the `erc721`
    /// reference.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - ID of a token.
    /// * `erc721` - Read access to a contract providing [`IErc721`] interface.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[selector(name = "tokenURI")]
    /// pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
    ///     Ok(self.metadata.token_uri(token_id, &self.erc721)?)
    /// }
    /// ```
    pub fn token_uri(
        &self,
        token_id: U256,
        erc721: &impl IErc721<Error = Error>,
    ) -> Result<String, Error> {
        erc721.owner_of(token_id)?;

        let base_uri = self.base_uri();

        let token_uri = if base_uri.is_empty() {
            String::new()
        } else {
            base_uri + &token_id.to_string()
        };

        Ok(token_uri)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::{Erc721Metadata, IErc165, IErc721Metadata};

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721Metadata as IErc721Metadata>::INTERFACE_ID;
        let expected = 0x93254542;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(<Erc721Metadata as IErc165>::supports_interface(
            0x5b5e139f.into()
        ));
    }
}
