//! Optional URI Metadata of the ERC-1155 standard, as defined
//! in the [ERC].
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions

use alloc::string::String;

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::stylus_proc::{public, sol_storage};

use crate::utils::introspection::erc165::{Erc165, IErc165};

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
    fn uri(&self, id: U256) -> String {
        let _ = id;
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
    // use crate::token::erc1155::extensions::{Erc1155MetadataUri,
    // IErc1155MetadataUri};

    // TODO: IErc1155MetadataUri should be refactored to have same api as
    // solidity  has:  https://github.com/OpenZeppelin/openzeppelin-contracts/blob/4764ea50750d8bda9096e833706beba86918b163/contracts/token/ERC1155/extensions/IErc1155MetadataUri.sol#L12
    // [motsu::test]
    // fn interface_id() {
    //     let actual = <Erc1155MetadataUri as
    // IErc1155MetadataUri>::INTERFACE_ID;     let expected = 0x5b5e139f;
    //     assert_eq!(actual, expected);
    // }
}
