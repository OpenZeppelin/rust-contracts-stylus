//! Implementation of nonce tracking for addresses.
//!
//! Nonces will only increment.

use alloc::vec::Vec;

use alloy_primitives::{uint, Address, U256};
use stylus_sdk::{
    prelude::storage,
    storage::{StorageMap, StorageU256},
    stylus_proc::{public, SolidityError},
};

use crate::utils::math::storage::AddAssignChecked;

const ONE: U256 = uint!(1_U256);

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// The nonce used for an `account` is not the expected current nonce.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error InvalidAccountNonce(address account, uint256 current_nonce);
    }
}

/// A Nonces error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// The nonce used for an `account` is not the expected current nonce.
    InvalidAccountNonce(InvalidAccountNonce),
}

/// State of a [`Nonces`] Contract.
#[storage]
pub struct Nonces {
    /// Mapping from address to its nonce.
    #[allow(clippy::used_underscore_binding)]
    pub _nonces: StorageMap<Address, StorageU256>,
}

#[public]
impl Nonces {
    /// Returns the unused nonce for the given account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address for which to return the nonce.
    #[must_use]
    pub fn nonces(&self, owner: Address) -> U256 {
        self._nonces.get(owner)
    }
}

impl Nonces {
    /// Consumes a nonce for the given `account`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - The address for which to consume the nonce.
    ///
    /// # Panics
    ///
    /// * If the nonce for the given `owner` exceeds `U256::MAX`.
    pub fn use_nonce(&mut self, owner: Address) -> U256 {
        let nonce = self._nonces.get(owner);

        self._nonces
            .setter(owner)
            .add_assign_checked(ONE, "nonce should not exceed `U256::MAX`");

        nonce
    }

    /// Same as `use_nonce` but checking that the `nonce` is the next valid for
    /// the owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - The address for which to consume the nonce.
    /// * `nonce` - The nonce to consume.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidAccountNonce`] - Returns an error if the `nonce` is
    ///   not the next valid nonce for the owner.
    ///
    /// # Panics
    ///
    /// * If the nonce for the given `owner` exceeds `U256::MAX`.
    pub fn use_checked_nonce(
        &mut self,
        owner: Address,
        nonce: U256,
    ) -> Result<(), Error> {
        let current_nonce = self.use_nonce(owner);

        if nonce != current_nonce {
            return Err(Error::InvalidAccountNonce(InvalidAccountNonce {
                account: owner,
                current_nonce,
            }));
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::msg;

    use super::ONE;
    use crate::utils::nonces::{Error, Nonces};

    #[motsu::test]
    fn initiate_nonce(contract: Nonces) {
        assert_eq!(contract.nonces(msg::sender()), U256::ZERO);
    }

    #[motsu::test]
    fn use_nonce(contract: Nonces) {
        let owner = msg::sender();

        let use_nonce = contract.use_nonce(owner);
        assert_eq!(use_nonce, U256::ZERO);

        let nonce = contract.nonces(owner);
        assert_eq!(nonce, ONE);
    }

    #[motsu::test]
    fn use_checked_nonce(contract: Nonces) {
        let owner = msg::sender();

        let use_checked_nonce = contract.use_checked_nonce(owner, U256::ZERO);
        assert!(use_checked_nonce.is_ok());

        let nonce = contract.nonces(owner);
        assert_eq!(nonce, ONE);
    }

    #[motsu::test]
    fn use_checked_nonce_invalid_nonce(contract: Nonces) {
        let owner = msg::sender();

        let use_checked_nonce = contract.use_checked_nonce(owner, ONE);
        assert!(matches!(
            use_checked_nonce,
            Err(Error::InvalidAccountNonce(_))
        ));
    }
}
