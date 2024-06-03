//! Optional Burnable extension of the ERC-20 standard.

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::erc20::{Erc20, Error};

/// Extension of [`Erc20`] that allows token holders to destroy both
/// their own tokens and those that they have an allowance for,
/// in a way that can be recognized off-chain (via event analysis).
pub trait IErc20Burnable {
    /// Destroys a `value` amount of tokens from the caller.
    /// lowering the total supply.
    ///
    /// Relies on the `update` mechanism.
    ///
    /// # Arguments
    ///
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn burn(&mut self, value: U256) -> Result<(), Error>;

    /// Destroys a `value` amount of tokens from `account`,
    /// lowering the total supply.
    ///
    /// Relies on the `update` mechanism.
    ///
    /// # Arguments
    ///
    /// * `account` - Owner's address.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// If not enough allowance is available, then the error
    /// [`Error::InsufficientAllowance`] is returned.
    /// If the `from` address is `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If the `from` address doesn't have enough tokens, then the error
    /// [`Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn burn_from(&mut self, account: Address, value: U256)
        -> Result<(), Error>;
}

impl IErc20Burnable for Erc20 {
    fn burn(&mut self, value: U256) -> Result<(), Error> {
        self._burn(msg::sender(), value)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Error> {
        self._spend_allowance(account, msg::sender(), value)?;
        self._burn(account, value)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::msg;

    use super::IErc20Burnable;
    use crate::erc20::{Erc20, Error, IErc20};

    #[grip::test]
    fn burns(contract: Erc20) {
        let zero = U256::ZERO;
        let one = U256::from(1);

        assert_eq!(zero, contract.total_supply());

        // Mint some tokens for msg::sender().
        let sender = msg::sender();

        let two = U256::from(2);
        contract._update(Address::ZERO, sender, two).unwrap();
        assert_eq!(two, contract.balance_of(sender));
        assert_eq!(two, contract.total_supply());

        contract.burn(one).unwrap();

        assert_eq!(one, contract.balance_of(sender));
        assert_eq!(one, contract.total_supply());
    }

    #[grip::test]
    fn burns_errors_when_insufficient_balance(contract: Erc20) {
        let zero = U256::ZERO;
        let one = U256::from(1);
        let sender = msg::sender();

        assert_eq!(zero, contract.balance_of(sender));

        let result = contract.burn(one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[grip::test]
    fn burn_from(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let sender = msg::sender();

        // Alice approves `msg::sender`.
        let one = U256::from(1);
        contract._allowances.setter(alice).setter(sender).set(one);

        // Mint some tokens for Alice.
        let two = U256::from(2);
        contract._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.balance_of(alice));
        assert_eq!(two, contract.total_supply());

        contract.burn_from(alice, one).unwrap();

        assert_eq!(one, contract.balance_of(alice));
        assert_eq!(one, contract.total_supply());
        assert_eq!(U256::ZERO, contract.allowance(alice, sender));
    }

    #[grip::test]
    fn burns_from_errors_when_insufficient_balance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // Alice approves `msg::sender`.
        let zero = U256::ZERO;
        let one = U256::from(1);

        contract._allowances.setter(alice).setter(msg::sender()).set(one);
        assert_eq!(zero, contract.balance_of(alice));

        let one = U256::from(1);

        let result = contract.burn_from(alice, one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[grip::test]
    fn burns_from_errors_when_invalid_sender(contract: Erc20) {
        let one = U256::from(1);

        contract
            ._allowances
            .setter(Address::ZERO)
            .setter(msg::sender())
            .set(one);

        let result = contract.burn_from(Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidSender(_))));
    }

    #[grip::test]
    fn burns_from_errors_when_insufficient_allowance(contract: Erc20) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // Mint some tokens for Alice.
        let one = U256::from(1);
        contract._update(Address::ZERO, alice, one).unwrap();
        assert_eq!(one, contract.balance_of(alice));

        let result = contract.burn_from(alice, one);
        assert!(matches!(result, Err(Error::InsufficientAllowance(_))));
    }
}
