//! Capped Contract.
//!
//! Extension of ERC-20 standard that adds a cap to the supply of tokens.
//!
//! Note that they will not be capped by simply including this module,
//! but only once the checks are put in place.

use alloc::{vec, vec::Vec};

use alloy_primitives::U256;
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{call::MethodError, prelude::*, storage::StorageU256};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates an error related to the operation that failed
        /// because `total_supply` exceeded the `cap`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20ExceededCap(uint256 increased_supply, uint256 cap);

        /// Indicates an error related to the operation that failed
        /// because the supplied `cap` is not a valid cap value.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InvalidCap(uint256 cap);
    }
}

/// A Capped error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the operation that failed
    /// because `total_supply` exceeded the `cap`.
    ExceededCap(ERC20ExceededCap),
    /// Indicates an error related to the operation that failed
    /// because the supplied `cap` is not a valid cap value.
    InvalidCap(ERC20InvalidCap),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of a [`Capped`] Contract.
#[storage]
pub struct Capped {
    /// A cap to the supply of tokens.
    pub(crate) cap: StorageU256,
}

/// Interface for the token supply cap logic.
#[interface_id]
pub trait ICapped {
    /// Returns the cap on the token's total supply.
    #[must_use]
    fn cap(&self) -> U256;
}

#[public]
#[implements(ICapped)]
impl Capped {
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `cap` - The token supply cap.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidCap`] - If cap is [`U256::ZERO`].
    #[constructor]
    pub fn constructor(&mut self, cap: U256) -> Result<(), Error> {
        if cap.is_zero() {
            return Err(Error::InvalidCap(ERC20InvalidCap { cap }));
        }
        self.cap.set(cap);
        Ok(())
    }
}

#[public]
impl ICapped for Capped {
    fn cap(&self) -> U256 {
        self.cap.get()
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address};
    use motsu::prelude::*;
    use stylus_sdk::prelude::*;

    use super::*;

    unsafe impl TopLevelStorage for Capped {}

    #[motsu::test]
    fn cap_works(contract: Contract<Capped>, alice: Address) {
        let value = uint!(2024_U256);
        contract
            .sender(alice)
            .constructor(value)
            .motsu_expect("should set cap");
        assert_eq!(contract.sender(alice).cap(), value);

        let value = U256::ONE;
        contract
            .sender(alice)
            .constructor(value)
            .motsu_expect("should set cap");
        assert_eq!(contract.sender(alice).cap(), value);

        let value = U256::ZERO;
        let err = contract
            .sender(alice)
            .constructor(value)
            .motsu_expect_err("should return error");
        assert!(matches!(
            err,
            Error::InvalidCap(ERC20InvalidCap { cap }) if cap == value
        ));
    }
}
