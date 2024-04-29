//! ERC-20 Capped extension.
//!
//! Extension of ERC-20 token that adds a cap
//! to the supply of tokens.

use crate::{
    erc20::{IERC20Virtual, IERC20},
    utils::capped::ICapped,
};

/// This macro provides an implementation of the ERC-20 Capped extension.
///
/// It adds all the functions from the `ICapped` trait, and expects the token
/// to contain `erc20` as a field that implements `IERC20Capped` trait.
///
/// Used to export interface for Stylus smart contracts with a single
/// `#[external]` macro.
#[macro_export]
macro_rules! erc20_capped_impl {
    () => {
        /// Returns the cap on the token's total supply.
        ///
        /// See [`ICapped::cap`].
        pub(crate) fn cap(&self) -> alloy_primitives::U256 {
            self.erc20.cap()
        }

        /// Sets the cap on the token's total supply.
        ///
        /// See [`ICapped::set_cap`].
        pub(crate) fn set_cap(
            &mut self,
            cap: alloy_primitives::U256,
        ) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.set_cap(cap).map_err(|e| e.into())
        }
    };
}

/// Interface for ERC-20 Capped extension.
#[allow(clippy::module_name_repetitions)]
pub trait IERC20Capped: IERC20Virtual + IERC20 + ICapped {}
