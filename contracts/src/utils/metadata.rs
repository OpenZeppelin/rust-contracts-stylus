//! Common Metadata Smart Contract.
use alloc::{string::String, vec, vec::Vec};

use stylus_sdk::{prelude::*, storage::StorageString};

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
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `name` - Token name.
    /// * `symbol` - Token symbol.
    #[constructor]
    pub fn constructor(&mut self, name: String, symbol: String) {
        self.name.set_str(name);
        self.symbol.set_str(symbol);
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
