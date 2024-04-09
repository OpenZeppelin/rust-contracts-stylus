//! Optional Burnable extension of the ERC-20 standard.
#[macro_export]
/// This macro provides implementation of ERC-20 Burnable extension.
/// It adds `burn` and `burn_from` function.
macro_rules! derive_erc20_burnable {
    () => {
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
        pub(crate) fn burn(&mut self, value: U256) -> Result<(), Error> {
            self.erc20._burn(msg::sender(), value)
        }

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
        /// * If the `from` address is `Address::ZERO`, then the error
        /// [`Error::InvalidSender`] is returned.
        /// If the `from` address doesn't have enough tokens, then the error
        /// [`Error::InsufficientBalance`] is returned.
        ///
        /// # Events
        ///
        /// Emits a [`Transfer`] event.
        pub(crate) fn burn_from(
            &mut self,
            account: Address,
            value: U256,
        ) -> Result<(), Error> {
            self.erc20._spend_allowance(account, msg::sender(), value)?;
            self.erc20._burn(account, value)
        }
    };
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::{msg, prelude::*};

    use crate::erc20::{Error, ERC20};

    sol_storage! {
        pub struct TestERC20Burnable {
            ERC20 erc20;
        }
    }

    #[external]
    #[inherit(ERC20)]
    impl TestERC20Burnable {
        derive_erc20_burnable!();
    }

    impl Default for TestERC20Burnable {
        fn default() -> Self {
            Self { erc20: ERC20::default() }
        }
    }

    #[grip::test]
    fn burns(contract: TestERC20Burnable) {
        let zero = U256::ZERO;
        let one = U256::from(1);

        assert_eq!(zero, contract.erc20.total_supply());

        // Mint some tokens for msg::sender().
        let sender = msg::sender();

        let two = U256::from(2);
        contract.erc20._update(Address::ZERO, sender, two).unwrap();
        assert_eq!(two, contract.erc20.balance_of(sender));
        assert_eq!(two, contract.erc20.total_supply());

        contract.burn(one).unwrap();

        assert_eq!(one, contract.erc20.balance_of(sender));
        assert_eq!(one, contract.erc20.total_supply());
    }

    #[grip::test]
    fn burns_errors_when_insufficient_balance(contract: TestERC20Burnable) {
        let one = U256::from(1);
        let sender = msg::sender();

        assert_eq!(U256::ZERO, contract.erc20.balance_of(sender));

        let result = contract.burn(one);

        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[grip::test]
    fn burn_from(contract: TestERC20Burnable) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
        let sender = msg::sender();

        // Alice approves `msg::sender`.
        let one = U256::from(1);
        contract.erc20._allowances.setter(alice).setter(sender).set(one);

        // Mint some tokens for Alice.
        let two = U256::from(2);
        contract.erc20._update(Address::ZERO, alice, two).unwrap();
        assert_eq!(two, contract.erc20.balance_of(alice));
        assert_eq!(two, contract.erc20.total_supply());

        contract.burn_from(alice, one).unwrap();

        assert_eq!(one, contract.erc20.balance_of(alice));
        assert_eq!(one, contract.erc20.total_supply());
        assert_eq!(U256::ZERO, contract.erc20.allowance(alice, sender));
    }

    #[grip::test]
    fn burns_from_errors_when_insufficient_balance(
        contract: TestERC20Burnable,
    ) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // Alice approves `msg::sender`.
        let one = U256::from(1);
        contract.erc20._allowances.setter(alice).setter(msg::sender()).set(one);
        assert_eq!(U256::ZERO, contract.erc20.balance_of(alice));

        let one = U256::from(1);
        let result = contract.burn_from(alice, one);
        assert!(matches!(result, Err(Error::InsufficientBalance(_))));
    }

    #[grip::test]
    fn burns_from_errors_when_invalid_sender(contract: TestERC20Burnable) {
        let one = U256::from(1);
        contract
            .erc20
            ._allowances
            .setter(Address::ZERO)
            .setter(msg::sender())
            .set(one);
        let result = contract.burn_from(Address::ZERO, one);
        assert!(matches!(result, Err(Error::InvalidSender(_))));
    }

    #[grip::test]
    fn burns_from_errors_when_insufficient_allowance(
        contract: TestERC20Burnable,
    ) {
        let alice = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

        // Mint some tokens for Alice.
        let one = U256::from(1);
        contract.erc20._update(Address::ZERO, alice, one).unwrap();
        assert_eq!(one, contract.erc20.balance_of(alice));

        let result = contract.burn_from(alice, one);
        assert!(matches!(result, Err(Error::InsufficientAllowance(_))));
    }
}
