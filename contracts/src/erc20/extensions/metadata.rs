//! Optional Metadata of the ERC-20 standard.

use alloc::string::String;

use stylus_proc::{external, sol_storage};

/// Number of decimals used by default on implementors of [`Metadata`].
pub const DEFAULT_DECIMALS: u8 = 18;

use crate::utils::Metadata;

sol_storage! {
    /// Metadata of the ERC20 token.
    ///
    /// It has hardcoded `decimals` to [`DEFAULT_DECIMALS`].
    pub struct ERC20Metadata {
        /// Common Metadata.
        Metadata _metadata
    }
}

// FIXME: Apply multi-level inheritance to export Metadata's functions.
// With the current version of SDK it is not possible.
// See https://github.com/OffchainLabs/stylus-sdk-rs/pull/120
#[external]
impl ERC20Metadata {
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
    ///
    /// # Panics
    ///
    /// * If the contract is already initialized, then this function panics.
    /// This ensures the contract is constructed only once.
    pub fn constructor(&mut self, name: String, symbol: String) {
        self._metadata.constructor(name, symbol);
    }

    /// Returns the name of the token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn name(&self) -> String {
        self._metadata.name()
    }

    /// Returns the symbol of the token, usually a shorter version of the name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn symbol(&self) -> String {
        self._metadata.symbol()
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

#[cfg(all(test, feature = "std"))]
mod tests {

    use super::{ERC20Metadata, DEFAULT_DECIMALS};
    use crate::utils::Metadata;

    impl Default for ERC20Metadata {
        fn default() -> Self {
            Self { _metadata: Metadata::default() }
        }
    }

    #[grip::test]
    fn constructs(meta: ERC20Metadata) {
        let name = meta.name();
        let symbol = meta.symbol();
        let decimals = meta.decimals();
        let initialized = meta._metadata._initialized.get();
        assert_eq!(name, "");
        assert_eq!(symbol, "");
        assert_eq!(decimals, DEFAULT_DECIMALS);
        assert_eq!(initialized, false);

        const NAME: &str = "Meta";
        const SYMBOL: &str = "Symbol";
        meta.constructor(NAME.to_owned(), SYMBOL.to_owned());

        let name = meta.name();
        let symbol = meta.symbol();
        let decimals = meta.decimals();
        let initialized = meta._metadata._initialized.get();
        assert_eq!(name, NAME);
        assert_eq!(symbol, SYMBOL);
        assert_eq!(decimals, DEFAULT_DECIMALS);
        assert_eq!(initialized, true);
    }

    #[grip::test]
    #[should_panic = "Metadata has already been initialized"]
    fn constructs_only_once(meta: ERC20Metadata) {
        const NAME: &str = "Meta";
        const SYMBOL: &str = "Symbol";
        meta.constructor(NAME.to_owned(), SYMBOL.to_owned());

        meta.constructor("Invalid".to_owned(), "Invalid".to_owned());
    }
}
