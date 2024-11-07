//! Optional Metadata of the ERC-721 standard.

use alloc::string::{String, ToString};

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::stylus_proc::{public, sol_storage};

use crate::{
    token::erc721::{Error, IErc721},
    utils::{introspection::erc165::IErc165, Metadata},
};

sol_storage! {
    /// Metadata of an [`crate::token::erc721::Erc721`] token.
    pub struct Erc721Metadata {
        /// Common Metadata.
        Metadata _metadata;
        /// Base URI for tokens.
        string _base_uri;
    }
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

impl IErc165 for Erc721Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        // NOTE: interface id is calculated using additional selector
        //  [`Erc721Metadata::token_uri`]
        0x_5b5e139f == u32::from_be_bytes(*interface_id)
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
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Id of a token.
    /// * `erc721` - Read access to a contract providing [`IErc721`] interface.
    ///
    /// # Errors
    ///
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    ///
    /// NOTE: In order to have [`Erc721Metadata::token_uri`] exposed in ABI,
    /// you need to do this manually.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// #[selector(name = "tokenURI")]
    /// pub fn token_uri(&self, token_id: U256) -> Result<String, Vec<u8>> {
    ///     Ok(self.metadata.token_uri(token_id, &self.erc721)?)
    /// }
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
    // use crate::token::erc721::extensions::{Erc721Metadata, IErc721Metadata};

    // TODO: IErc721Metadata should be refactored to have same api as solidity
    //  has:  https://github.com/OpenZeppelin/openzeppelin-contracts/blob/4764ea50750d8bda9096e833706beba86918b163/contracts/token/ERC721/extensions/IERC721Metadata.sol#L12
    // [motsu::test]
    // fn interface_id() {
    //     let actual = <Erc721Metadata as IErc721Metadata>::INTERFACE_ID;
    //     let expected = 0x5b5e139f;
    //     assert_eq!(actual, expected);
    // }
}
