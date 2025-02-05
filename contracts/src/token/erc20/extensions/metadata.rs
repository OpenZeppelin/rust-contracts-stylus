//! Optional Metadata of the ERC-20 standard.

use alloc::string::String;

use alloy_primitives::FixedBytes;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::stylus_proc::{public, storage};

use crate::utils::introspection::erc165::IErc165;

/// Number of decimals used by default on implementors of [`Metadata`].
pub const DEFAULT_DECIMALS: u8 = 18;

use alloc::vec::Vec;

use crate::utils::Metadata;

/// State of an [`Erc20Metadata`] contract.
#[storage]
pub struct Erc20Metadata {
    /// [`Metadata`] contract.
    #[allow(clippy::used_underscore_binding)]
    pub _metadata: Metadata,
}

/// Interface for the optional metadata functions from the ERC-20 standard.
#[interface_id]
pub trait IErc20Metadata {
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
    fn decimals(&self) -> u8;
}

#[public]
impl IErc20Metadata for Erc20Metadata {
    fn name(&self) -> String {
        self._metadata.name()
    }

    fn symbol(&self) -> String {
        self._metadata.symbol()
    }

    fn decimals(&self) -> u8 {
        DEFAULT_DECIMALS
    }
}

impl IErc165 for Erc20Metadata {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc20Metadata>::INTERFACE_ID
            == u32::from_be_bytes(*interface_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::token::erc20::extensions::{Erc20Metadata, IErc20Metadata};

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc20Metadata as IErc20Metadata>::INTERFACE_ID;
        let expected = 0xa219a025;
        assert_eq!(actual, expected);
    }
}
