//! ERC-20 Pausable extension.
//!
//! Provides ability to pause token transfers.

use crate::{
    erc20::{IERC20Virtual, IERC20},
    utils::pausable::IPausable,
};

/// This macro provides an implementation of the ERC-20 Pausable extension.
///
/// It adds all the functions from the `IPausable` trait, and expects the token
/// to contain `erc20` as a field that implements `IERC20Pausable` trait.
///
/// Used to export interface for Stylus smart contracts with a single
/// `#[external]` macro.
#[macro_export]
macro_rules! erc20_pausable_impl {
    () => {
        /// Returns true if the contract is paused, and false otherwise.
        ///
        /// See [`IPausable::paused`].
        pub(crate) fn paused(&self) -> bool {
            self.erc20.paused()
        }

        /// Triggers `Paused` state.
        ///
        /// See [`IPausable::pause`].
        pub(crate) fn pause(&mut self) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.pause().map_err(|e| e.into())
        }

        /// Triggers `Unpaused` state.
        ///
        /// See [`IPausable::unpause`].
        pub(crate) fn unpause(&mut self) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.unpause().map_err(|e| e.into())
        }

        /// Modifier to make a function callable
        /// only when the contract is NOT paused.
        ///
        /// See [`IPausable::when_not_paused`].
        pub(crate) fn when_not_paused(
            &self,
        ) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.when_not_paused().map_err(|e| e.into())
        }

        /// Modifier to make a function callable
        /// only when the contract is paused.
        ///
        /// See [`IPausable::when_paused`].
        pub(crate) fn when_paused(&self) -> Result<(), alloc::vec::Vec<u8>> {
            self.erc20.when_paused().map_err(|e| e.into())
        }
    };
}

/// Interface for ERC-20 Pausable extension.
///
/// It provides ERC-20 token with pausable token transfers,
/// minting and burining.
///
/// Useful for scenarios such as preventing trades until the end of an
/// evaluation period, or having an emergency switch for freezing all token
/// transfers in the event of a large bug.
#[allow(clippy::module_name_repetitions)]
pub trait IERC20Pausable: IERC20Virtual + IERC20 + IPausable {}
