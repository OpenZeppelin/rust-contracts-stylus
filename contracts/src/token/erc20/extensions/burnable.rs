//! Optional Burnable extension of the ERC-20 standard.

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::token::erc20::{self, Erc20};

/// Extension of [`Erc20`] that allows token holders to destroy both
/// their own tokens and those that they have an allowance for,
/// in a way that can be recognized off-chain (via event analysis).
pub trait IErc20Burnable {
    /// The error type associated to this ERC-20 Burnable trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Destroys a `value` amount of tokens from the caller, lowering the total
    /// supply.
    ///
    /// # Arguments
    ///
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`erc20::Error::InsufficientBalance`] - If the `from` address doesn't
    ///   have enough tokens.
    ///
    /// # Events
    ///
    /// * [`erc20::Transfer`].
    fn burn(&mut self, value: U256) -> Result<(), Self::Error>;

    /// Destroys a `value` amount of tokens from `account`, lowering the total
    /// supply.
    ///
    /// # Arguments
    ///
    /// * `account` - Owner's address.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`erc20::Error::InsufficientAllowance`] - If not enough allowance is
    ///   available.
    /// * [`erc20::Error::InvalidSender`] - If the `from` address is
    ///   `Address::ZERO`.
    /// * [`erc20::Error::InsufficientBalance`] - If the `from` address doesn't
    ///   have enough tokens.
    ///
    /// # Events
    ///
    /// * [`erc20::Transfer`].
    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Self::Error>;
}

impl IErc20Burnable for Erc20 {
    type Error = erc20::Error;

    fn burn(&mut self, value: U256) -> Result<(), Self::Error> {
        self._burn(msg::sender(), value)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        self._spend_allowance(account, msg::sender(), value)?;
        self._burn(account, value)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use super::IErc20Burnable;
    use crate::token::erc20::{Erc20, Error, IErc20};

    #[motsu::test]
    fn burns(contract: Erc20) {
        let zero = U256::ZERO;
        let one = uint!(1_U256);

        assert_eq!(zero, contract.total_supply());

        // Mint some tokens for msg::sender().
        let sender = msg::sender();

        let two = uint!(2_U256);
        contract._update(Address::ZERO, sender, two).unwrap();
        assert_eq!(two, contract.balance_of(sender));
        assert_eq!(two, contract.total_supply());

        contract.burn(one).unwrap();

        assert_eq!(one, contract.balance_of(sender));
        assert_eq!(one, contract.total_supply());
    }

    #[motsu::test]
    fn burns_errors_when_insufficient_balance(contract: Erc20) {
        let zero = U256::ZERO;
        let one = uint!(1_U256);
        let sender = msg::sender();

        assert_eq!(zero, contract.balance_of(sender));

        let result = contract.burn(one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[motsu::test]
    fn burn_from(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let sender = msg::sender();

        // Alice approves `msg::sender`.
        let one = uint!(1_U256);
        contract._allowances.setter(alice).setter(sender).set(one);

        // Mint some tokens for Alice.
        let two = uint!(2_U256);
        contract._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.balance_of(alice));
        assert_eq!(two, contract.total_supply());

        contract.burn_from(alice, one).unwrap();

        assert_eq!(one, contract.balance_of(alice));
        assert_eq!(one, contract.total_supply());
        assert_eq!(U256::ZERO, contract.allowance(alice, sender));
    }

    #[motsu::test]
    fn burns_from_errors_when_insufficient_balance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // Alice approves `msg::sender`.
        let zero = U256::ZERO;
        let one = uint!(1_U256);

        contract._allowances.setter(alice).setter(msg::sender()).set(one);
        assert_eq!(zero, contract.balance_of(alice));

        let one = uint!(1_U256);

        let result = contract.burn_from(alice, one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[motsu::test]
    fn burns_from_errors_when_invalid_approver(contract: Erc20) {
        let one = uint!(1_U256);

        contract
            ._allowances
            .setter(Address::ZERO)
            .setter(msg::sender())
            .set(one);

        let result = contract.burn_from(Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidApprover(_))));
    }

    #[motsu::test]
    fn burns_from_errors_when_insufficient_allowance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // Mint some tokens for Alice.
        let one = uint!(1_U256);
        contract._update(Address::ZERO, alice, one).unwrap();
        assert_eq!(one, contract.balance_of(alice));

        let result = contract.burn_from(alice, one);
        assert!(matches!(result, Err(Error::InsufficientAllowance(_))));
    }
}
