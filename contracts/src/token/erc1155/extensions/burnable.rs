//! Extension of {ERC1155} that allows token holders to destroy both their
//! own tokens and those that they have been approved to use.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};

/// Extension of [`Erc1155`] that allows token holders to destroy both their
/// own tokens and those that they have been approved to use.
pub trait IErc1155Burnable {
    /// The error type associated to this ERC-1155 burnable trait
    /// implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Extension of {ERC1155} that allows token holders to destroy both their
    /// own tokens and those that they have been approved to use.
    ///
    /// The approval is cleared when the token is burned. Relies on the `_burn`
    /// mechanism.
    ///
    /// # Arguments
    ///
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// If the caller is not account address and the account has not been
    /// approved, then the error [`Error::MissingApprovalForAll`] is
    /// returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    /// * The caller or account must own `token_id` or be an approved operator.
    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Extension of {ERC1155} that allows token holders to destroy both their
    /// own tokens and those that they have been approved to use.
    ///
    /// The approval is cleared when the token is burned. Relies on the `_burn`
    /// mechanism.
    ///
    /// # Arguments
    ///
    /// * `values` - All amount to be burnt.
    /// * `ids` - All amount to be burnt.
    ///
    /// # Errors
    ///
    /// If the caller is not account address and the account has not been
    /// approved, then the error [`Error::MissingApprovalForAll`] is
    /// returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    /// * The caller or account must own `token_id` or be an approved operator.
    fn burn_batch(
        &mut self,
        account: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Self::Error>;
}
