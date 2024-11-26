//! Optional URI Metadata of the ERC-1155 standard, as defined
//! in the [ERC].
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions

use alloc::string::String;

use alloy_primitives::{FixedBytes, U256};
use alloy_sol_macro::sol;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::stylus_proc::{public, sol_storage};

use crate::utils::introspection::erc165::{Erc165, IErc165};

sol! {
    /// Emitted when the URI for token type `id` changes to `value`, if it is
    /// a non-programmatic URI.
    ///
    /// If a [`URI`] event was emitted for `id`, the standard [guarantees] that
    /// `value` will equal the value returned by [`IErc1155MetadataUri::uri`].
    ///
    /// [guarantees]: https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions
    #[allow(missing_docs)]
    event URI(string value, uint256 indexed id);
}

sol_storage! {
    /// URI Metadata of an [`crate::token::erc1155::Erc1155`] token.
    pub struct Erc1155MetadataUri {
        /// Used as the URI for all token types by relying on ID substitution,
        /// e.g. https://token-cdn-domain/{id}.json.
        string _uri;
    }
}

/// Interface for the optional metadata functions from the ERC-1155 standard.
#[interface_id]
pub trait IErc1155MetadataUri {
    /// Returns the URI for token type `id`.
    ///
    /// If the `\{id\}` substring is present in the URI, it must be replaced by
    /// clients with the actual token type ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id.
    fn uri(&self, id: U256) -> String;
}

// FIXME: Apply multi-level inheritance to export Metadata's functions.
// With the current version of SDK it is not possible.
// See https://github.com/OffchainLabs/stylus-sdk-rs/pull/120
#[public]
impl IErc1155MetadataUri for Erc1155MetadataUri {
    /// This implementation returns the same URI for *all* token types.
    /// Clients calling this function must replace the `\{id\}` substring with
    /// the actual token type ID.
    fn uri(&self, _id: U256) -> String {
        self._uri.get_string()
    }
}

impl IErc165 for Erc1155MetadataUri {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc1155MetadataUri>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

impl Erc1155MetadataUri {
    /// Sets a new URI for all token types, by relying on the token type ID
    /// substitution mechanism [defined in the ERC].
    ///
    /// [defined in the ERC]: https://eips.ethereum.org/EIPS/eip-1155#metadata
    ///
    /// By this mechanism, any occurrence of the `\{id\}` substring in either
    /// the URI or any of the values in the JSON file at said URI will be
    /// replaced by clients with the token type ID.
    ///
    /// For example, the `https://token-cdn-domain/\{id\}.json` URI would be
    /// interpreted by clients as
    /// `https://token-cdn-domain/000000000000000000000000000000000000000000000000000000000004cce0.json`
    /// for token type ID 0x4cce0.
    ///
    /// See [`Self::uri`].
    ///
    /// Because these URIs cannot be meaningfully represented by the
    /// [`crate::token::erc1155::URI`] event, this function emits no events.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_uri` - New URI value.
    pub fn _set_uri(&mut self, new_uri: String) {
        self._uri.set_str(new_uri);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;

    use super::{Erc1155MetadataUri, IErc1155MetadataUri};

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        U256::from(num)
    }

    #[motsu::test]
    fn uri_ignores_token_id(contract: Erc1155MetadataUri) {
        let uri = String::from("https://token-cdn-domain/\\{id\\}.json");
        contract._uri.set_str(uri.clone());

        let token_id = random_token_id();
        assert_eq!(uri, contract.uri(token_id));

        let token_id = random_token_id();
        assert_eq!(uri, contract.uri(token_id));
    }

    #[motsu::test]
    fn set_uri_works(contract: Erc1155MetadataUri) {
        let token_id = random_token_id();
        let old_uri = String::from("https://token-cdn-domain/\\{id\\}.json");
        contract._uri.set_str(old_uri.clone());
        assert_eq!(old_uri, contract.uri(token_id));

        let new_uri =
            String::from("https://new-token-cdn-domain/\\{id\\}.json");
        contract._set_uri(new_uri.clone());
        assert_eq!(new_uri, contract.uri(token_id));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc1155MetadataUri as IErc1155MetadataUri>::INTERFACE_ID;
        let expected = 0x0e89341c;
        assert_eq!(actual, expected);
    }
}
