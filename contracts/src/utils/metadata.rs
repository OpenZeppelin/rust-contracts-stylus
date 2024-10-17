//! Common Metadata Smart Contract.
use alloc::string::String;

use stylus_proc::{public, sol_storage};

sol_storage! {
    /// Metadata of the token.
    pub struct Metadata {
        /// Token name.
        string _name;
        /// Token symbol.
        string _symbol;
    }
}

/// Required interface of a [`Metadata`] compliant contract.
pub trait IMetadata {
    /// Returns the name of the token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn name(&self) -> String;

    /// Returns the symbol of the token, usually a shorter version of the name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn symbol(&self) -> String;
}

#[public]
impl IMetadata for Metadata {
    fn name(&self) -> String {
        self._name.get_string()
    }

    fn symbol(&self) -> String {
        self._symbol.get_string()
    }
}
