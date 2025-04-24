//! Optional Metadata of the ERC-721 standard.

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use alloy_primitives::{FixedBytes, U256};
use stylus_sdk::{prelude::*, storage::StorageString};

use crate::{
    token::erc721::{self, IErc721},
    utils::{
        introspection::erc165::{Erc165, IErc165},
        Metadata,
    },
};

/// State of an [`Erc721Metadata`] contract.
#[storage]
pub struct Erc721Metadata {
    /// [`Metadata`] contract.
    pub(crate) metadata: Metadata,
    // We keep this field public, since this is used to simulate overriding
    // (which is not possible in Rust).
    /// Base URI for tokens.
    pub base_uri: StorageString,
}

/// Interface for the optional metadata functions from the ERC-721 standard.
pub trait IErc721Metadata {
    // Manually calculated, as the trait is missing
    // [`Erc721Metadata::token_uri`].
    /// Solidity interface id associated with [`IErc721Metadata`] trait.
    /// Computed as a XOR of selectors for each function in the trait.
    const INTERFACE_ID: u32 =
        u32::from_be_bytes(stylus_sdk::function_selector!("name"))
            ^ u32::from_be_bytes(stylus_sdk::function_selector!("symbol"))
            ^ u32::from_be_bytes(stylus_sdk::function_selector!(
                "tokenURI", U256
            ));

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
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }
}

// TODO: uncomment once multiple public attributes are supported
// #[public]
impl Erc721Metadata {
    /// Constructor
    // #[constructor]
    pub fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }
}

impl IErc165 for Erc721Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc721Metadata>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
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
        self.base_uri.get_string()
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
    /// * [`erc721::Error::NonexistentToken`] - If the token does not exist.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[selector(name = "tokenURI")]
    /// pub fn token_uri(&self, token_id: U256) -> Result<String, erc721::Error> {
    ///     self.metadata.token_uri(token_id, &self.erc721)
    /// }
    /// ```
    pub fn token_uri(
        &self,
        token_id: U256,
        erc721: &impl IErc721<Error = erc721::Error>,
    ) -> Result<String, erc721::Error> {
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
        let expected = 0x5b5e139f;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(Erc721Metadata::supports_interface(
            <Erc721Metadata as IErc721Metadata>::INTERFACE_ID.into()
        ));
        assert!(Erc721Metadata::supports_interface(
            <Erc721Metadata as IErc165>::INTERFACE_ID.into()
        ));

        let fake_interface_id = 0x12345678u32;
        assert!(!Erc721Metadata::supports_interface(fake_interface_id.into()));
    }
}
