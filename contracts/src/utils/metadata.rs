//! Common Metadata Smart Contract.
use alloc::string::String;

use stylus_sdk::stylus_proc::{public, sol_storage};

sol_storage! {
    /// Metadata of the token.
    pub struct Metadata {
        /// Token name.
        string _name;
        /// Token symbol.
        string _symbol;
    }
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
