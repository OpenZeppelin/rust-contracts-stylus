//! Optional metadata of the ERC-20 standard.
use alloc::string::String;

use stylus_proc::{external, sol_storage};

/// Number of decimals used by default on implementors of [`Metadata`].
pub const DEFAULT_DECIMALS: u8 = 18;

sol_storage! {
    /// Optional metadata of the ERC-20 standard.
    pub struct Metadata {
        /// Token name.
        string _name;
        /// Token symbol.
        string _symbol;
    }
}

#[external]
impl Metadata {
    /// Initializes a [`Metadata`] instance with the passed `name` and
    /// `symbol`. It also sets `decimals` to [`DEFAULT_DECIMALS`].
    ///
    /// Note that there are no setters for these fields. This makes them
    /// immutable: they can only be set once at construction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `name` - The name of the token.
    /// * `symbol` - The symbol of the token.
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

    /// Returns the number of decimals used to get a user-friendly
    /// representation of values of this token.
    ///
    /// For example, if `decimals` equals `2`, a balance of `505` tokens should
    /// be displayed to a user as `5.05` (`505 / 10 ** 2`).
    ///
    /// Tokens usually opt for a value of `18`, imitating the relationship
    /// between Ether and Wei. This is the default value returned by this
    /// function ([`DEFAULT_DECIMALS`]), unless it's overridden.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// NOTE: This information is only used for *display* purposes: in
    /// no way it affects any of the arithmetic of the contract, including
    /// [`ERC20::balance_of`] and [`ERC20::transfer`].
    pub fn decimals(&self) -> u8 {
        // TODO: Use `U8` an avoid the conversion once https://github.com/OffchainLabs/stylus-sdk-rs/issues/117
        // gets resolved.
        DEFAULT_DECIMALS
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::storage::{StorageString, StorageType};

    use super::{Metadata, DEFAULT_DECIMALS};
    #[allow(unused_imports)]
    use crate::test_utils;

    impl Default for Metadata {
        fn default() -> Self {
            let root = U256::ZERO;
            Metadata {
                _name: unsafe { StorageString::new(root, 0) },
                _symbol: unsafe {
                    StorageString::new(root + U256::from(32), 0)
                },
            }
        }
    }

    #[test]
    fn constructs() {
        test_utils::with_storage::<Metadata>(|meta| {
            let name = meta.name();
            let symbol = meta.symbol();
            let decimals = meta.decimals();
            assert_eq!(name, "");
            assert_eq!(symbol, "");
            assert_eq!(decimals, 0);

            const NAME: &str = "Meta";
            const SYMBOL: &str = "Symbol";

            meta.constructor(NAME.to_owned(), SYMBOL.to_owned());

            let name = meta.name();
            let symbol = meta.symbol();
            let decimals = meta.decimals();
            assert_eq!(name, NAME);
            assert_eq!(symbol, SYMBOL);
            assert_eq!(decimals, DEFAULT_DECIMALS);
        })
    }
}
