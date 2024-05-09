//! Optional Metadata of the ERC-721 standard.

use alloc::string::String;

use stylus_proc::{external, sol_storage};

use crate::utils::Metadata;

sol_storage! {
    /// Metadata of the ERC-721 token.
    pub struct ERC721Metadata {
        /// Common Metadata.
        Metadata _metadata;
        /// Base URI for tokens
        string _base_uri;
    }
}

// FIXME: Apply multi-level inheritance to export Metadata's functions.
// With the current version of SDK it is not possible.
// See https://github.com/OffchainLabs/stylus-sdk-rs/pull/120
#[external]
impl ERC721Metadata {
    /// Initializes a [`Metadata`] instance with the passed `name`,
    /// `symbol`, and `base_uri`.
    ///
    /// Note that there are no setters for these fields. This makes them
    /// immutable: they can only be set once at construction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `name` - Token collection name.
    /// * `symbol` - Token collection symbol.
    /// * `base_uri` - Base of URI for tokens' collection.
    ///
    /// # Panics
    ///
    /// * If the contract is already initialized, then this function panics.
    /// This ensures the contract is constructed only once.
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        base_uri: String,
    ) {
        self._metadata.constructor(name, symbol);
        self._base_uri.set_str(base_uri);
    }

    /// Returns the token collection name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn name(&self) -> String {
        self._metadata.name()
    }

    /// Returns token collection symbol.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn symbol(&self) -> String {
        self._metadata.symbol()
    }

    /// Returns the base of Uniform Resource Identifier (URI) for tokens'
    /// collection.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn base_uri(&self) -> String {
        self._base_uri.get_string()
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::{prelude::StorageType, storage::StorageString};

    use super::ERC721Metadata;
    use crate::utils::Metadata;

    impl Default for ERC721Metadata {
        fn default() -> Self {
            let root = U256::ZERO;
            Self {
                _metadata: Metadata::default(),
                _base_uri: unsafe {
                    StorageString::new(root + U256::from(96), 0)
                },
            }
        }
    }
    #[grip::test]
    fn constructs(meta: ERC721Metadata) {
        let name = meta.name();
        let symbol = meta.symbol();
        let initialized = meta._metadata._initialized.get();
        assert_eq!(name, "");
        assert_eq!(symbol, "");
        assert_eq!(initialized, false);

        const NAME: &str = "Meta";
        const SYMBOL: &str = "Symbol";
        const BASE_URI: &str = "URI";
        meta.constructor(
            NAME.to_owned(),
            SYMBOL.to_owned(),
            BASE_URI.to_owned(),
        );

        let name = meta.name();
        let symbol = meta.symbol();
        let initialized = meta._metadata._initialized.get();
        let base_uri = meta.base_uri();
        assert_eq!(name, NAME);
        assert_eq!(symbol, SYMBOL);
        assert_eq!(initialized, true);
        assert_eq!(base_uri, BASE_URI);
    }

    #[grip::test]
    #[should_panic = "Metadata has already been initialized"]
    fn constructs_only_once(meta: ERC721Metadata) {
        const NAME: &str = "Meta";
        const SYMBOL: &str = "Symbol";
        const BASE_URI: &str = "URI";

        meta.constructor(
            NAME.to_owned(),
            SYMBOL.to_owned(),
            BASE_URI.to_owned(),
        );

        meta.constructor(
            "Invalid".to_owned(),
            "Invalid".to_owned(),
            "Invalid".to_owned(),
        );
    }
}
