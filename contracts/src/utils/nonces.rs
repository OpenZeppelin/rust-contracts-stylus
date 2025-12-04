//! Implementation of nonce tracking for addresses.
//!
//! Nonces will only increment.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
pub use sol::*;
use stylus_sdk::{
    call::MethodError,
    prelude::*,
    storage::{StorageMap, StorageU256},
};

use crate::utils::math::storage::AddAssignChecked;

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

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of a [`Nonces`] Contract.
#[storage]
pub struct Nonces {
    /// Mapping from address to its nonce.
    pub(crate) nonces: StorageMap<Address, StorageU256>,
}

/// Interface for [`Nonces`]
pub trait INonces {
    /// Returns the unused nonce for the given account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - The address for which to return the nonce.
    #[must_use]
    fn nonces(&self, owner: Address) -> U256;
}

#[public]
#[implements(INonces)]
impl Nonces {}

#[public]
impl INonces for Nonces {
    fn nonces(&self, owner: Address) -> U256 {
        self.nonces.get(owner)
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
    /// * If the nonce for the given `owner` exceeds [`U256::MAX`].
    pub fn use_nonce(&mut self, owner: Address) -> U256 {
        let nonce = self.nonces.get(owner);

        self.nonces.setter(owner).add_assign_checked(
            U256::ONE,
            "nonce should not exceed `U256::MAX`",
        );

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
    /// * If the nonce for the given `owner` exceeds [`U256::MAX`].
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::*;

    use super::*;
    use crate::utils::nonces::{Error, INonces, Nonces};

    unsafe impl TopLevelStorage for Nonces {}

    #[motsu::test]
    fn initiate_nonce(contract: Contract<Nonces>, alice: Address) {
        assert_eq!(contract.sender(alice).nonces(alice), U256::ZERO);
    }

    #[motsu::test]
    fn use_nonce(contract: Contract<Nonces>, alice: Address) {
        let use_nonce = contract.sender(alice).use_nonce(alice);
        assert_eq!(use_nonce, U256::ZERO);

        let nonce = contract.sender(alice).nonces(alice);
        assert_eq!(nonce, U256::ONE);
    }

    #[motsu::test]
    fn use_checked_nonce(contract: Contract<Nonces>, alice: Address) {
        let use_checked_nonce =
            contract.sender(alice).use_checked_nonce(alice, U256::ZERO);
        assert!(use_checked_nonce.is_ok());

        let nonce = contract.sender(alice).nonces(alice);
        assert_eq!(nonce, U256::ONE);
    }

    #[motsu::test]
    fn use_checked_nonce_invalid_nonce(
        contract: Contract<Nonces>,
        alice: Address,
    ) {
        let use_checked_nonce =
            contract.sender(alice).use_checked_nonce(alice, U256::ONE);
        assert!(matches!(
            use_checked_nonce,
            Err(
                Error::InvalidAccountNonce(InvalidAccountNonce {
                account,
                current_nonce,
            })) if account == alice && current_nonce.is_zero()

        ));
    }
}
