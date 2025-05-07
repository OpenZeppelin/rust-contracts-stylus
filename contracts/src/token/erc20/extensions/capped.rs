//! Capped Contract.
//!
//! Extension of ERC-20 standard that adds a cap to the supply of tokens.
//!
//! Note that they will not be capped by simply including this module,
//! but only once the checks are put in place.

use alloc::vec;

use alloy_primitives::U256;
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

#[public]
impl Capped {
    /// Returns the cap on the token's total supply.
    #[must_use]
    pub fn cap(&self) -> U256 {
        self.cap.get()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{uint, Address};
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::TopLevelStorage;

    use super::Capped;

    unsafe impl TopLevelStorage for Capped {}

    #[motsu::test]
    fn cap_works(contract: Contract<Capped>, alice: Address) {
        let value = uint!(2024_U256);
        contract.init(alice, |contract| contract.cap.set(value));
        assert_eq!(contract.sender(alice).cap(), value);

        let value = uint!(1_U256);
        contract.init(alice, |contract| contract.cap.set(value));
        assert_eq!(contract.sender(alice).cap(), value);
    }
}
