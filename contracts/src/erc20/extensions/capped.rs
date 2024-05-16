//! Capped Contract.
//!
//! Extension of [`ERC20`] that adds a cap to the supply of tokens.
//!
//! Note that they will not be capped by simply including this module,
//! but only once the checks are put in place.

use alloy_primitives::U256;
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};

sol_storage! {
    /// State of a Capped Contract.
    #[allow(clippy::pub_underscore_fields)]
    pub struct Capped {
        /// A cap to the supply of tokens.
        uint256 _cap;
        /// This field should be unnecessary once constructors are supported in
        /// the SDK.
        bool _initialized

    }
}

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

#[external]
impl Capped {
    /// Initializes a [`Capped`] instance
    /// with the passed `cap`.
    ///
    /// Note that there is no setter for this field. This makes it
    /// immutable: it can only be set once at construction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `cap` - Value of the `cap`.
    ///
    ///  # Errors
    ///
    /// * If `cap` value is invalid [`U256::ZERO`], then the error
    /// [`Error::InvalidCap`] is returned.
    ///
    /// # Panics
    ///
    /// * If the contract is already initialized, then this function panics.
    /// This ensures the contract is constructed only once.
    pub fn constructor(&mut self, cap: U256) -> Result<(), Error> {
        let is_initialized = self._initialized.get();
        assert!(!is_initialized, "Capped has already been initialized");

        if cap.is_zero() {
            return Err(Error::InvalidCap(ERC20InvalidCap { cap }));
        }

        self._cap.set(cap);
        self._initialized.set(true);
        Ok(())
    }

    /// Returns the cap on the token's total supply.
    pub fn cap(&self) -> U256 {
        self._cap.get()
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::storage::{StorageBool, StorageType, StorageU256};

    use super::{Capped, Error};

    impl Default for Capped {
        fn default() -> Self {
            let root = U256::ZERO;
            Capped {
                _cap: unsafe { StorageU256::new(root, 0) },
                _initialized: unsafe {
                    StorageBool::new(root + U256::from(32), 0)
                },
            }
        }
    }

    #[grip::test]
    fn constructs(capped: Capped) {
        let initialized = capped._initialized.get();
        assert_eq!(initialized, false);

        let cap = U256::from(2024);
        capped
            .constructor(cap)
            .expect("Capped::constructor should work for proper `cap` value");

        let initialized = capped._initialized.get();
        assert_eq!(cap, capped.cap());
        assert_eq!(initialized, true);
    }

    #[grip::test]
    #[should_panic = "Capped has already been initialized"]
    fn constructs_only_once(capped: Capped) {
        capped
            .constructor(U256::from(2024))
            .expect("Capped::constructor should work for proper `cap` value");

        capped.constructor(U256::from(4048)).unwrap();
    }

    #[grip::test]
    fn constructor_error_when_invalid_cap(capped: Capped) {
        let initialized = capped._initialized.get();
        assert_eq!(initialized, false);

        let cap = U256::ZERO;
        let result = capped.constructor(cap);
        assert!(matches!(result, Err(Error::InvalidCap(_))));
    }

    #[grip::test]
    fn cap_works(contract: Capped) {
        let value = U256::from(2024);
        contract._cap.set(value);
        assert_eq!(contract.cap(), value);

        let value = U256::from(1);
        contract._cap.set(value);
        assert_eq!(contract.cap(), value);
    }
}
