//! Common Metadata Smart Contract.
use alloc::{string::String, vec, vec::Vec};

use stylus_sdk::{prelude::*, storage::StorageString, stylus_proc::public};

use crate::utils::introspection::erc165::{Erc165, IErc165};

use alloy_primitives::FixedBytes;

/// State of a [`Metadata`] contract.
#[storage]
pub struct Metadata {
    /// Token name.
    pub(crate) name: StorageString,
    /// Token symbol.
    pub(crate) symbol: StorageString,
}

#[public]
impl Metadata {
    /// Returns the name of the token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn name(&self) -> String {
        self.name.get_string()
    }

    /// Returns the symbol of the token, usually a shorter version of the name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn symbol(&self) -> String {
        self.symbol.get_string()
    }
}

impl IErc165 for Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc165::supports_interface(interface_id)
    }
}
