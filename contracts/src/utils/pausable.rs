//! Pausable Contract.
//!
//! Contract module which allows implementing an emergency stop mechanism
//! that can be triggered by an authorized account.
//!
//! It provides functions [`Pausable::when_not_paused`]
//! and [`Pausable::when_paused`],
//! which can be added to the functions of your contract.
//!
//! Note that they will not be pausable by simply including this module,
//! only once the modifiers are put in place.

use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{evm, msg};

sol! {
    /// Emitted when pause is triggered by `account`.
    #[allow(missing_docs)]
    event Paused(address account);

    /// Emitted when the pause is lifted by `account`.
    #[allow(missing_docs)]
    event Unpaused(address account);
}

sol! {
    /// Indicates an error related to the operation that failed
    /// because the contract is paused.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error EnforcedPause();

    /// Indicates an error related to the operation that failed
    /// because the contract is not paused.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ExpectedPause();
}

/// A Pausable error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the operation that failed
    /// because the contract had been in `Paused` state.
    EnforcedPause(EnforcedPause),
    /// Indicates an error related to the operation that failed
    /// because the contract had been in `Unpaused` state.
    ExpectedPause(ExpectedPause),
}

sol_storage! {
    /// State of a Pausable Contract.
    #[allow(missing_docs)]
    pub struct Pausable {
        /// Indicates whether the contract is `Paused`.
        bool _paused;
        /// Initialization marker. If true this means that the constructor was
        /// called.
        ///
        /// This field should be unnecessary once constructors are supported in
        /// the SDK.
        bool _initialized;
    }
}

#[external]
impl Pausable {
    /// Initializes a [`Pausable`] contract with the passed `paused`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `paused` - Indicates if contract is paused.
    ///
    /// # Panics
    ///
    /// * If the contract is already initialized, then this function panics.
    /// This ensures the contract is constructed only once.
    pub fn constructor(&mut self, paused: bool) {
        let is_initialized = self._initialized.get();
        assert!(!is_initialized, "Pausable has already been initialized");

        self._paused.set(paused);
        self._initialized.set(true);
    }

    /// Returns true if the contract is paused, and false otherwise.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn paused(&self) -> bool {
        self._paused.get()
    }

    /// Triggers `Paused` state.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    fn pause(&mut self) -> Result<(), Error> {
        self.when_not_paused()?;
        self._paused.set(true);
        evm::log(Paused { account: msg::sender() });
        Ok(())
    }

    /// Triggers `Unpaused` state.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Unpaused` state, then the error
    /// [`Error::ExpectedPause`] is returned.
    fn unpause(&mut self) -> Result<(), Error> {
        self.when_paused()?;
        self._paused.set(false);
        evm::log(Unpaused { account: msg::sender() });
        Ok(())
    }

    /// Modifier to make a function callable
    /// only when the contract is NOT paused.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    fn when_not_paused(&self) -> Result<(), Error> {
        if self._paused.get() {
            return Err(Error::EnforcedPause(EnforcedPause {}));
        }
        Ok(())
    }

    /// Modifier to make a function callable
    /// only when the contract is paused.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Unpaused` state, then the error
    /// [`Error::ExpectedPause`] is returned.
    fn when_paused(&self) -> Result<(), Error> {
        if !self._paused.get() {
            return Err(Error::ExpectedPause(ExpectedPause {}));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::storage::{StorageBool, StorageType};

    use crate::utils::pausable::{Error, Pausable};

    impl Default for Pausable {
        fn default() -> Self {
            let root = U256::ZERO;
            Pausable {
                _paused: unsafe { StorageBool::new(root, 0) },
                _initialized: unsafe {
                    StorageBool::new(root + U256::from(32), 0)
                },
            }
        }
    }

    #[grip::test]
    fn constructs(pausable: Pausable) {
        assert_eq!(false, pausable._initialized.get());

        let paused = false;
        pausable.constructor(paused);

        assert_eq!(paused, pausable._paused.get());
        assert_eq!(true, pausable._initialized.get());
    }

    #[grip::test]
    #[should_panic = "Pausable has already been initialized"]
    fn constructs_only_once(pausable: Pausable) {
        let paused = false;
        pausable.constructor(paused);
        pausable.constructor(paused);
    }

    #[grip::test]
    fn paused_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);
        // Check for paused
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);
    }

    #[grip::test]
    fn when_not_paused_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        let result = contract.when_not_paused();
        assert!(result.is_ok());
    }

    #[grip::test]
    fn when_not_paused_errors_when_paused(contract: Pausable) {
        // Check for paused
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        let result = contract.when_not_paused();
        assert!(matches!(result, Err(Error::EnforcedPause(_))));
    }

    #[grip::test]
    fn when_paused_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        let result = contract.when_paused();
        assert!(result.is_ok());
    }

    #[grip::test]
    fn when_paused_errors_when_not_paused(contract: Pausable) {
        // Check for paused
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        let result = contract.when_paused();
        assert!(matches!(result, Err(Error::ExpectedPause(_))));
    }

    #[grip::test]
    fn pause_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        // Pause the contract
        contract.pause().expect("Pause action must work in unpaused state");
        assert_eq!(contract.paused(), true);
    }

    #[grip::test]
    fn pause_errors_when_already_paused(contract: Pausable) {
        // Check for paused
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        // Pause the paused contract
        let result = contract.pause();
        assert!(matches!(result, Err(Error::EnforcedPause(_))));
        assert_eq!(contract.paused(), true);
    }

    #[grip::test]
    fn unpause_works(contract: Pausable) {
        // Check for paused
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        // Unpause the paused contract
        contract.unpause().expect("Unpause action must work in paused state");
        assert_eq!(contract.paused(), false);
    }

    #[grip::test]
    fn unpause_errors_when_already_unpaused(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        // Unpause the unpaused contract
        let result = contract.unpause();
        assert!(matches!(result, Err(Error::ExpectedPause(_))));
        assert_eq!(contract.paused(), false);
    }
}
