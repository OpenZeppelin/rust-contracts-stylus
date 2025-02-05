//! Common Metadata Smart Contract.
use alloc::{string::String, vec::Vec};

use stylus_sdk::{
    prelude::storage, storage::StorageString, stylus_proc::public,
};

/// State of a [`Metadata`] contract.
#[storage]
pub struct Metadata {
    /// Token name.
    #[allow(clippy::used_underscore_binding)]
    pub _name: StorageString,
    /// Token symbol.
    #[allow(clippy::used_underscore_binding)]
    pub _symbol: StorageString,
}

#[public]
impl Metadata {
    /// Returns the name of the token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn name(&self) -> String {
        self._name.get_string()
    }

    /// Returns the symbol of the token, usually a shorter version of the name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn symbol(&self) -> String {
        self._symbol.get_string()
    }
}
