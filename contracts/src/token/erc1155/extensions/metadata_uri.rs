//! Optional MetadataURI of the ERC-1155 standard.

use alloc::string::String;

use stylus_proc::{public, sol_storage};

sol_storage! {
    /// MetadataURI of an [`crate::token::erc1155::Erc1155`] token.
    pub struct Erc1155MetadataURI {
        /// Optional base URI for tokens.
        string _base_uri;
    }
}

/// Interface for the optional metadata functions from the ERC-1155 standard.
pub trait IErc1155MetadataURI {
    /// Returns the base of Uniform Resource Identifier (URI) for tokens'
    /// collection.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn base_uri(&self) -> String;
}

#[public]
impl IErc1155MetadataURI for Erc1155MetadataURI {
    fn base_uri(&self) -> String {
        self._base_uri.get_string()
    }
}
