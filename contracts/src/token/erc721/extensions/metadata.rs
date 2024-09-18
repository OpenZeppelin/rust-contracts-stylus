//! Optional Metadata of the ERC-721 standard.

use alloc::string::String;

use alloy_primitives::FixedBytes;
use openzeppelin_stylus_proc::interface_id;
use stylus_proc::{public, sol_storage};

use crate::{
    token::erc20::extensions::IErc20Metadata,
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

    /// Returns the base of Uniform Resource Identifier (URI) for tokens'
    /// collection.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn base_uri(&self) -> String;
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

    fn base_uri(&self) -> String {
        self._base_uri.get_string()
    }
}

impl IErc165 for Erc721Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc721Metadata>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
    }
}
