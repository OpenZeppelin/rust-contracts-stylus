//! Capped Contract.
//!
//! Contract module which allows implementing a cap mechanism.
//!
//! This module should be used through inheritance and wrappers.
//!
//! Note that they will not be capped by simply including this module,
//! only once the checks are put in place.

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{evm, msg};

sol_storage! {
    /// State of a Capped Contract.
    pub struct Capped {
        /// A cap to the supply of tokens.
        uint256 _cap;
    }
}

sol! {
    /// Emitted when `_cap` is set to `cap` value
    /// by an `account`.
    event Cap(address indexed account, uint256 cap);
}

sol! {
    /// Indicates an error related to the operation that failed
    /// because `total_supply` exceeded the `_cap`.
    #[derive(Debug)]
    error ExceededCap(uint256 increasedSupply, uint256 cap);
    /// Indicates an error related to the operation that failed
    /// because the supplied `cap` is not a valid cap value.
    #[derive(Debug)]
    error InvalidCap(uint256 cap);
}

/// A Capped error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the operation that failed
    /// because `total_supply` exceeded the `_cap`.
    ExceededCap(ExceededCap),
    /// Indicates an error related to the operation that failed
    /// because the supplied `cap` is not a valid cap value.
    InvalidCap(InvalidCap),
}

/// Interface of a `Capped` Contract.
#[allow(clippy::module_name_repetitions)]
pub trait ICapped {
    /// Returns the cap on the token's total supply.
    fn cap(&self) -> U256;

    /// Sets the cap on the token's total supply.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    fn set_cap(&mut self, cap: U256) -> Result<(), Error>;
}

/// Implementation of `ICapped` trait for Capped Contract.
#[external]
impl ICapped for Capped {
    fn cap(&self) -> U256 {
        self._cap.get()
    }

    fn set_cap(&mut self, cap: U256) -> Result<(), Error> {
        if cap.is_zero() {
            return Err(Error::InvalidCap(InvalidCap { cap }));
        }

        self._cap.set(cap);

        evm::log(Cap { account: msg::sender(), cap });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::storage::{StorageType, StorageU256};

    use super::{Capped, Error, ICapped};

    impl Default for Capped {
        fn default() -> Self {
            Capped { _cap: unsafe { StorageU256::new(U256::ZERO, 0) } }
        }
    }

    #[grip::test]
    fn cap_works(contract: Capped) {
        // Check `cap` value
        let value = U256::from(2024);
        contract._cap.set(value);
        assert_eq!(contract.cap(), value);

        let value = U256::from(1);
        contract._cap.set(value);
        assert_eq!(contract.cap(), value);
    }

    #[grip::test]
    fn set_cap_works(contract: Capped) {
        let initial_value = U256::from(1);
        contract._cap.set(initial_value);
        assert_eq!(contract.cap(), initial_value);

        // Set cap value
        let new_value = U256::from(2024);
        contract
            .set_cap(new_value)
            .expect("Set cap must work for proper `_cap` value");
        assert_eq!(contract.cap(), new_value);
    }

    #[grip::test]
    fn set_cap_when_invalid_cap(contract: Capped) {
        let initial_value = U256::from(1);
        contract._cap.set(initial_value);
        assert_eq!(contract.cap(), initial_value);

        // Try to set invalid cap value
        let result = contract.set_cap(U256::ZERO);
        assert!(matches!(result, Err(Error::InvalidCap(_))));
        assert_eq!(contract.cap(), initial_value);
    }
}
