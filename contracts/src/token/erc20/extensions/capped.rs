//! Capped Contract.
//!
//! Extension of ERC-20 standard that adds a cap to the supply of tokens.
//!
//! Note that they will not be capped by simply including this module,
//! but only once the checks are put in place.

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_sdk::stylus_proc::{public, sol_storage, SolidityError};

sol! {
    /// Indicates an error related to the operation that failed
    /// because `total_supply` exceeded the `_cap`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20ExceededCap(uint256 increased_supply, uint256 cap);

    /// Indicates an error related to the operation that failed
    /// because the supplied `cap` is not a valid cap value.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidCap(uint256 cap);
}

/// A Capped error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the operation that failed
    /// because `total_supply` exceeded the `_cap`.
    ExceededCap(ERC20ExceededCap),
    /// Indicates an error related to the operation that failed
    /// because the supplied `cap` is not a valid cap value.
    InvalidCap(ERC20InvalidCap),
}

sol_storage! {
    /// State of a Capped Contract.
    #[allow(clippy::pub_underscore_fields)]
    pub struct Capped {
        /// A cap to the supply of tokens.
        uint256 _cap;
    }
}

#[public]
impl Capped {
    /// Returns the cap on the token's total supply.
    pub fn cap(&self) -> U256 {
        self._cap.get()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::uint;

    use super::Capped;

    #[motsu::test]
    fn cap_works(contract: Capped) {
        let value = uint!(2024_U256);
        contract._cap.set(value);
        assert_eq!(contract.cap(), value);

        let value = uint!(1_U256);
        contract._cap.set(value);
        assert_eq!(contract.cap(), value);
    }
}
