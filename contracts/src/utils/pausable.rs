use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{evm, msg};

sol_storage! {
    pub struct Pausable {
        /// Indicates whether the contract is `Paused`
        bool _paused;
    }
}

sol! {
    /// Emitted when the pause is triggered by an account.
    event Paused(address account);
    /// Emitted when the unpause is lifted by an account.
    event Unpaused(address account);
}

sol! {
    /// The operation failed because the contract is in `Paused` state.
    #[derive(Debug)]
    error EnforcedPause();
    /// The operation failed because the contract is in `Unpaused` state.
    #[derive(Debug)]
    error ExpectedPause();
}

#[derive(SolidityError, Debug)]
pub enum Error {
    /// The operation failed because the contract is in `Paused` state.
    EnforcedPause(EnforcedPause),
    /// The operation failed because the contract is in `Unpaused` state.
    ExpectedPause(ExpectedPause),
}

pub trait IPausable {
    /// Returns true if the contract is paused, and false otherwise.
    fn paused(&self) -> Result<bool, Error>;

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
}

pub trait IPausableModifier {
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

impl IPausableModifier for Pausable {
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

// External methods
#[external]
impl IPausable for Pausable {
    fn paused(&self) -> Result<bool, Error> {
        Ok(self._paused.get())
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
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::storage::{StorageBool, StorageType};

    use crate::utils::pausable::{
        Error, IPausable, IPausableModifier, Pausable,
    };

    impl Default for Pausable {
        fn default() -> Self {
            Pausable { _paused: unsafe { StorageBool::new(U256::ZERO, 0) } }
        }
    }

    #[grip::test]
    fn paused_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        let unpaused_result = contract.paused().expect("Paused must work");
        assert_eq!(unpaused_result, false);
        // Check for paused
        contract._paused.set(true);
        let paused_result = contract.paused().expect("Paused must work");
        assert_eq!(paused_result, true);
    }

    #[grip::test]
    fn when_not_paused_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, false);

        let result = contract.when_not_paused();
        assert!(result.is_ok());
    }

    #[grip::test]
    fn when_not_paused_errors_when_paused(contract: Pausable) {
        // Check for paused
        contract._paused.set(true);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, true);

        let result = contract.when_not_paused();
        assert!(matches!(result, Err(Error::EnforcedPause(_))));
    }

    #[grip::test]
    fn when_paused_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(true);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, true);

        let result = contract.when_paused();
        assert!(result.is_ok());
    }

    #[grip::test]
    fn when_paused_errors_when_not_paused(contract: Pausable) {
        // Check for paused
        contract._paused.set(false);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, false);

        let result = contract.when_paused();
        assert!(matches!(result, Err(Error::ExpectedPause(_))));
    }

    #[grip::test]
    fn pause_works(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, false);

        // Pause the contract
        contract.pause().expect("Pause action must work in unpaused state");
        let paused_result = contract.paused().expect("Paused must work");
        assert_eq!(paused_result, true);
    }

    #[grip::test]
    fn pause_errors_when_already_paused(contract: Pausable) {
        // Check for paused
        contract._paused.set(true);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, true);

        // Pause the paused contract
        let result = contract.pause();
        let paused_result = contract.paused().expect("Paused must work");
        assert!(matches!(result, Err(Error::EnforcedPause(_))));
        assert_eq!(paused_result, true);
    }

    #[grip::test]
    fn unpause_works(contract: Pausable) {
        // Check for paused
        contract._paused.set(true);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, true);

        // Unpause the paused contract
        contract.unpause().expect("Unpause action must work in paused state");
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, false);
    }

    #[grip::test]
    fn unpause_errors_when_already_unpaused(contract: Pausable) {
        // Check for unpaused
        contract._paused.set(false);
        let result = contract.paused().expect("Paused must work");
        assert_eq!(result, false);

        // Unpause the unpaused contract
        let result = contract.unpause();
        let unpaused_result = contract.paused().expect("Paused must work");
        assert!(matches!(result, Err(Error::ExpectedPause(_))));
        assert_eq!(unpaused_result, false);
    }
}
