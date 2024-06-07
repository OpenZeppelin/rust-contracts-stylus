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
    #[cfg_attr(test, derive(motsu::StylusDefault))]
    pub struct Pausable {
        /// Indicates whether the contract is `Paused`.
        bool _paused;
    }
}

#[external]
impl Pausable {
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
    /// If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    pub fn pause(&mut self) -> Result<(), Error> {
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
    /// If the contract is in `Unpaused` state, then the error
    /// [`Error::ExpectedPause`] is returned.
    pub fn unpause(&mut self) -> Result<(), Error> {
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
    /// If the contract is in `Paused` state, then the error
    /// [`Error::EnforcedPause`] is returned.
    pub fn when_not_paused(&self) -> Result<(), Error> {
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
    /// If the contract is in `Unpaused` state, then the error
    /// [`Error::ExpectedPause`] is returned.
    pub fn when_paused(&self) -> Result<(), Error> {
        if !self._paused.get() {
            return Err(Error::ExpectedPause(ExpectedPause {}));
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::U256;
    use stylus_sdk::storage::{StorageBool, StorageType};

    use crate::utils::pausable::{Error, Pausable};

    #[motsu::test]
    fn paused_works(contract: Pausable) {
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);
    }

    #[motsu::test]
    fn when_not_paused_works(contract: Pausable) {
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        let result = contract.when_not_paused();
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn when_not_paused_errors_when_paused(contract: Pausable) {
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        let result = contract.when_not_paused();
        assert!(matches!(result, Err(Error::EnforcedPause(_))));
    }

    #[motsu::test]
    fn when_paused_works(contract: Pausable) {
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        let result = contract.when_paused();
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn when_paused_errors_when_not_paused(contract: Pausable) {
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        let result = contract.when_paused();
        assert!(matches!(result, Err(Error::ExpectedPause(_))));
    }

    #[motsu::test]
    fn pause_works(contract: Pausable) {
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        // Pause the contract
        let res = contract.pause();
        assert!(res.is_ok());
        assert_eq!(contract.paused(), true);
    }

    #[motsu::test]
    fn pause_errors_when_already_paused(contract: Pausable) {
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        let result = contract.pause();
        assert!(matches!(result, Err(Error::EnforcedPause(_))));
        assert_eq!(contract.paused(), true);
    }

    #[motsu::test]
    fn unpause_works(contract: Pausable) {
        contract._paused.set(true);
        assert_eq!(contract.paused(), true);

        // Unpause the paused contract
        let res = contract.unpause();
        assert!(res.is_ok());
        assert_eq!(contract.paused(), false);
    }

    #[motsu::test]
    fn unpause_errors_when_already_unpaused(contract: Pausable) {
        contract._paused.set(false);
        assert_eq!(contract.paused(), false);

        // Unpause the unpaused contract
        let result = contract.unpause();
        assert!(matches!(result, Err(Error::ExpectedPause(_))));
        assert_eq!(contract.paused(), false);
    }
}
