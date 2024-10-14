//! Optional Metadata of the ERC-721 standard.

use alloc::string::String;

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_proc::sol_storage;

use crate::{
    token::erc721::{Erc721, Error},
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

    /// Returns the token URI for `token_id`.
    ///
    /// NOTE: Don't forget to add `#[selector(name = "tokenURI")]` while
    /// reexporting, since actual solidity name is different.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    #[selector(name = "tokenURI")]
    fn token_uri(&self, token_id: U256) -> Result<String, Error>;
}

impl IErc165 for Erc721Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Erc721 as IErc721Metadata>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::token::erc721::{extensions::IErc721Metadata, Erc721};

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721 as IErc721Metadata>::INTERFACE_ID;
        let expected = 0x5b5e139f;
        assert_eq!(actual, expected);
    }
}
