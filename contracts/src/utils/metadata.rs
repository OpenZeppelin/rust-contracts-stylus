//! Common Metadata Smart Contract.
use alloc::{string::String, vec::Vec};

use stylus_sdk::{
    prelude::storage, storage::StorageString, stylus_proc::public,
};

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
    /// Constructor
    #[constructor]
    pub fn constructor(&mut self, name: String, symbol: String) {
        self._name.set_str(name);
        self._symbol.set_str(symbol);
    }

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
