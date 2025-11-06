//! Pausable Contract.
//!
//! Contract module which allows implementing an emergency stop mechanism
//! that can be triggered by an authorized account.
//!
//! It provides functions [`Pausable::when_not_paused`]
//! and [`Pausable::when_paused`],
//! which can be added to the functions of your contract.
//!
//! Note that your contract will not be pausable by simply including this
//! module, only once and where you use [`Pausable::when_not_paused`].
//!
//! Note that [`Pausable::pause`] and [`Pausable::unpause`] methods are not
//! exposed by default.
//! You should expose them manually in your contract's abi.

use alloc::{vec, vec::Vec};

pub use sol::*;
use stylus_sdk::{
    call::MethodError, evm, msg, prelude::*, storage::StorageBool,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when pause is triggered by `account`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Paused(address account);

        /// Emitted when the pause is lifted by `account`.
        #[derive(Debug)]
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

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of a [`Pausable`] Contract.
#[storage]
pub struct Pausable {
    /// Indicates whether the contract is `Paused`.
    pub(crate) paused: StorageBool,
}

/// Pausable interface.
pub trait IPausable {
    /// Returns true if the contract is paused, and false otherwise.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn paused(&self) -> bool;
}

#[public]
#[implements(IPausable)]
impl Pausable {}

#[public]
impl IPausable for Pausable {
    fn paused(&self) -> bool {
        self.paused.get()
    }
}

impl Pausable {
    /// Triggers `Paused` state.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::EnforcedPause`] - If the contract is in `Paused` state.
    pub fn pause(&mut self) -> Result<(), Error> {
        self.when_not_paused()?;
        self.paused.set(true);
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
    /// * [`Error::ExpectedPause`] - If the contract is in `Unpaused` state.
    pub fn unpause(&mut self) -> Result<(), Error> {
        self.when_paused()?;
        self.paused.set(false);
        evm::log(Unpaused { account: msg::sender() });
        Ok(())
    }

    /// Helper to make a function callable only when the contract is NOT
    /// paused.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::EnforcedPause`] - If the contract is in the `Paused` state.
    pub fn when_not_paused(&self) -> Result<(), Error> {
        if self.paused.get() {
            return Err(Error::EnforcedPause(EnforcedPause {}));
        }
        Ok(())
    }

    /// Helper to make a function callable
    /// only when the contract is paused.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// * [`Error::ExpectedPause`] - If the contract is in `Unpaused` state.
    pub fn when_paused(&self) -> Result<(), Error> {
        if !self.paused.get() {
            return Err(Error::ExpectedPause(ExpectedPause {}));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::Address;
    use motsu::prelude::{Contract, ResultExt};
    use stylus_sdk::prelude::*;

    use super::*;

    unsafe impl TopLevelStorage for Pausable {}

    #[motsu::test]
    fn paused_works(contract: Contract<Pausable>, alice: Address) {
        contract.sender(alice).paused.set(true);
        assert!(contract.sender(alice).paused());

        contract.sender(alice).paused.set(false);
        assert!(!contract.sender(alice).paused());
    }

    #[motsu::test]
    fn when_not_paused_works(contract: Contract<Pausable>, alice: Address) {
        contract.sender(alice).paused.set(false);

        let result = contract.sender(alice).when_not_paused();
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn when_not_paused_errors_when_paused(
        contract: Contract<Pausable>,
        alice: Address,
    ) {
        contract.sender(alice).paused.set(true);
        assert!(contract.sender(alice).paused());

        let result = contract.sender(alice).when_not_paused();
        assert!(matches!(result, Err(Error::EnforcedPause(EnforcedPause {}))));
    }

    #[motsu::test]
    fn when_paused_works(contract: Contract<Pausable>, alice: Address) {
        contract.sender(alice).pause().motsu_unwrap();
        assert!(contract.sender(alice).paused());

        let result = contract.sender(alice).when_paused();
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn when_paused_errors_when_not_paused(
        contract: Contract<Pausable>,
        alice: Address,
    ) {
        contract.sender(alice).paused.set(false);
        assert!(!contract.sender(alice).paused());

        let result = contract.sender(alice).when_paused();
        assert!(matches!(result, Err(Error::ExpectedPause(ExpectedPause {}))));
    }

    #[motsu::test]
    fn pause_works(contract: Contract<Pausable>, alice: Address) {
        contract.sender(alice).paused.set(false);
        assert!(!contract.sender(alice).paused());

        // Pause the contract
        let res = contract.sender(alice).pause();
        assert!(res.is_ok());
        assert!(contract.sender(alice).paused());
    }

    #[motsu::test]
    fn pause_errors_when_already_paused(
        contract: Contract<Pausable>,
        alice: Address,
    ) {
        contract.sender(alice).paused.set(true);
        assert!(contract.sender(alice).paused());

        let result = contract.sender(alice).pause();
        assert!(matches!(result, Err(Error::EnforcedPause(EnforcedPause {}))));
        assert!(contract.sender(alice).paused());
    }

    #[motsu::test]
    fn unpause_works(contract: Contract<Pausable>, alice: Address) {
        contract.sender(alice).paused.set(true);
        assert!(contract.sender(alice).paused());

        // Unpause the paused contract
        let res = contract.sender(alice).unpause();
        assert!(res.is_ok());
        assert!(!contract.sender(alice).paused());
    }

    #[motsu::test]
    fn unpause_errors_when_already_unpaused(
        contract: Contract<Pausable>,
        alice: Address,
    ) {
        contract.sender(alice).paused.set(false);
        assert!(!contract.sender(alice).paused());

        // Unpause the unpaused contract
        let result = contract.sender(alice).unpause();
        assert!(matches!(result, Err(Error::ExpectedPause(ExpectedPause {}))));
        assert!(!contract.sender(alice).paused());
    }
}
