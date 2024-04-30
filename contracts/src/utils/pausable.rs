//! Pausable Contract.
//!
//! Contract module which allows implementing an emergency stop
//! mechanism that can be triggered by an authorized account.
//!
//! This module should be used through inheritance and wrappers.
//! It will make available the modifiers `IPausable::when_not_paused`
//! and `IPausable::when_paused`, which can be applied to the functions
//! of your contract.
//!
//! Note that they will not be pausable by simply including this module,
//! only once the modifiers are put in place.

use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{evm, msg};

sol_storage! {
    /// State of a Pausable Contract.
    #[allow(clippy::pub_underscore_fields)]
    pub struct Pausable {
        /// Indicates whether the contract is `Paused`.
        bool _paused;
    }
}

sol! {
    /// Emitted when the `Pause` is triggered by an `account`.
    #[allow(missing_docs)]
    event Paused(address indexed account);

    /// Emitted when the `Unpause` is lifted by an `account`.
    #[allow(missing_docs)]
    event Unpaused(address indexed account);
}

sol! {
    /// Indicates an error related to the operation that failed
    /// because the contract had been in `Paused` state.
    #[derive(Debug)]
    error EnforcedPause();

    /// Indicates an error related to the operation that failed
    /// because the contract had been in `Unpaused` state.
    #[derive(Debug)]
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

/// Interface of a `Pausable` Contract.
#[allow(clippy::module_name_repetitions)]
pub trait IPausable {
    /// Returns true if the contract is paused, and false otherwise.
    fn paused(&self) -> bool;

    /// Triggers `Paused` state.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    fn pause(&mut self) -> Result<(), Error>;

    /// Triggers `Unpaused` state.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Unpaused` state, then the error
    /// [`Error::ExpectedPause`] is returned.
    fn unpause(&mut self) -> Result<(), Error>;

    /// Modifier to make a function callable
    /// only when the contract is NOT paused.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    fn when_not_paused(&self) -> Result<(), Error>;

    /// Modifier to make a function callable
    /// only when the contract is paused.
    ///
    /// # Errors
    ///
    /// * If the contract is in `Unpaused` state, then the error
    /// [`Error::ExpectedPause`] is returned.
    fn when_paused(&self) -> Result<(), Error>;
}

/// Implementation of `IPausable` trait for Pausable Contract.
#[external]
impl IPausable for Pausable {
    fn paused(&self) -> bool {
        self._paused.get()
    }

    fn pause(&mut self) -> Result<(), Error> {
        self.when_not_paused()?;
        self._paused.set(true);
        evm::log(Paused { account: msg::sender() });
        Ok(())
    }

    fn unpause(&mut self) -> Result<(), Error> {
        self.when_paused()?;
        self._paused.set(false);
        evm::log(Unpaused { account: msg::sender() });
        Ok(())
    }

    fn when_not_paused(&self) -> Result<(), Error> {
        if self._paused.get() {
            return Err(Error::EnforcedPause(EnforcedPause {}));
        }
        Ok(())
    }

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

    use crate::utils::pausable::{Error, IPausable, Pausable};

    impl Default for Pausable {
        fn default() -> Self {
            Pausable { _paused: unsafe { StorageBool::new(U256::ZERO, 0) } }
        }
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
