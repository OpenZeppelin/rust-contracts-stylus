//! Optional URI Metadata of the ERC-1155 standard, as defined
//! in the [ERC].
//!
//! [ERC]: https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions

use alloc::{string::String, vec, vec::Vec};

use alloy_primitives::{FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{prelude::*, storage::StorageString};

use crate::utils::introspection::erc165::{Erc165, IErc165};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the URI for token type `id` changes to `value`, if it is
        /// a non-programmatic URI.
        ///
        /// If a [`URI`] event was emitted for `id`, the standard [guarantees] that
        /// `value` will equal the value returned by [`IErc1155MetadataUri::uri`].
        ///
        /// [guarantees]: https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions
        #[derive(Debug)]
        #[allow(missing_docs)]
        event URI(string value, uint256 indexed id);
    }
}

/// State of an [`Erc1155MetadataUri`] contract.
#[storage]
pub struct Erc1155MetadataUri {
    /// Used as the URI for all token types by relying on ID substitution,
    /// e.g. https://token-cdn-domain/{id}.json.
    pub(crate) uri: StorageString,
}

/// Interface for the optional metadata functions from the ERC-1155 standard.
#[interface_id]
pub trait IErc1155MetadataUri {
    /// Returns the URI for token type `id`.
    ///
    /// If the `id` substring is present in the URI, it must be replaced by
    /// clients with the actual token type ID.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id.
    fn uri(&self, id: U256) -> String;
}

#[public]
impl IErc1155MetadataUri for Erc1155MetadataUri {
    /// This implementation returns the same URI for all token types.
    /// Clients calling this function must replace the `id` substring with
    /// the actual token type ID.
    fn uri(&self, _id: U256) -> String {
        self.uri.get_string()
    }
}

impl IErc165 for Erc1155MetadataUri {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc1155MetadataUri>::interface_id()
            == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::Contract;
    use stylus_sdk::{alloy_primitives::uint, prelude::TopLevelStorage};

    use super::{Erc1155MetadataUri, IErc1155MetadataUri, IErc165};

    unsafe impl TopLevelStorage for Erc1155MetadataUri {}

    #[motsu::test]
    fn uri_ignores_token_id(
        contract: Contract<Erc1155MetadataUri>,
        alice: Address,
    ) {
        let uri = String::from("https://token-cdn-domain/\\{id\\}.json");
        contract.init(alice, |contract| {
            contract.uri.set_str(uri.clone());
        });

        let token_id = uint!(1_U256);
        assert_eq!(uri, contract.sender(alice).uri(token_id));

        let token_id = uint!(2_U256);
        assert_eq!(uri, contract.sender(alice).uri(token_id));
    }

    #[motsu::test]
    fn interface_id() {
        let actual =
            <Erc1155MetadataUri as IErc1155MetadataUri>::interface_id();
        let expected = 0x0e89341c;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(Erc1155MetadataUri::supports_interface(
            <Erc1155MetadataUri as IErc1155MetadataUri>::interface_id().into()
        ));
        assert!(Erc1155MetadataUri::supports_interface(
            <Erc1155MetadataUri as IErc165>::interface_id().into()
        ));

        let fake_interface_id = 0x12345678u32;
        assert!(!Erc1155MetadataUri::supports_interface(
            fake_interface_id.into()
        ));
    }
}
