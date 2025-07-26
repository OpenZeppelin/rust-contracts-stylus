//! Optional Metadata of the ERC-20 standard.

use alloc::string::String;

use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    alloy_primitives::{uint, U8},
    storage::StorageString,
};

use crate::token::erc20::IErc20;

/// Number of decimals used by default on implementors of [`Metadata`].
pub const DEFAULT_DECIMALS: U8 = uint!(18_U8);

/// Interface for the optional metadata functions from the ERC-20 standard.
#[interface_id]
pub trait IErc20Metadata: IErc20 + Erc20MetadataStorage {
    /// Returns the name of the token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn name(&self) -> String {
        Erc20MetadataStorage::name(self).get_string()
    }

    /// Returns the symbol of the token, usually a shorter version of the name.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn symbol(&self) -> String {
        Erc20MetadataStorage::symbol(self).get_string()
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
    /// NOTE: This information is only used for *display* purposes: in
    /// no way it affects any of the arithmetic of the contract, including
    /// [`super::super::IErc20::balance_of`] and
    /// [`super::super::IErc20::transfer`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[must_use]
    fn decimals(&self) -> U8 {
        DEFAULT_DECIMALS
    }
}

/// Storage trait for the ERC-20 Metadata.
pub trait Erc20MetadataStorage {
    /// Return the balances of the token.
    fn name(&self) -> &StorageString;
    /// Return the allowances of the token.
    fn symbol(&self) -> &StorageString;
}
