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
    /// * `&mut self` - Write access to the contract's state.
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
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Owner's address.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`erc20::Error::InsufficientAllowance`] - If not enough allowance is
    ///   available.
    /// * [`erc20::Error::InvalidSender`] - If the `from` address is
    ///   [`Address::ZERO`].
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::*;

    use super::IErc20Burnable;
    use crate::token::erc20::{
        ERC20InsufficientAllowance, ERC20InsufficientBalance, Erc20, Error,
        IErc20,
    };

    #[motsu::test]
    fn burns(contract: Contract<Erc20>, alice: Address) {
        let one = U256::ONE;
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply());

        // Mint some tokens for Alice.

        let two = uint!(2_U256);
        contract
            .sender(alice)
            ._update(Address::ZERO, alice, two)
            .motsu_unwrap();
        assert_eq!(two, contract.sender(alice).balance_of(alice));
        assert_eq!(two, contract.sender(alice).total_supply());

        contract.sender(alice).burn(one).motsu_unwrap();

        assert_eq!(one, contract.sender(alice).balance_of(alice));
        assert_eq!(one, contract.sender(alice).total_supply());
    }

    #[motsu::test]
    fn burns_errors_when_insufficient_balance(
        contract: Contract<Erc20>,
        alice: Address,
    ) {
        let one = U256::ONE;
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice));

        let result = contract.sender(alice).burn(one);
        assert!(matches!(
            result,
            Err(
                Error::InsufficientBalance(ERC20InsufficientBalance { sender, balance, needed })
            )if sender == alice && balance.is_zero() && needed == one,
        ));
    }

    #[motsu::test]
    fn burn_from(contract: Contract<Erc20>, alice: Address, bob: Address) {
        // Alice approves `msg::sender`.
        let one = U256::ONE;
        contract.sender(alice).approve(bob, one).motsu_unwrap();

        // Mint some tokens for Alice.
        let two = uint!(2_U256);
        contract
            .sender(alice)
            ._update(Address::ZERO, alice, two)
            .motsu_unwrap();
        assert_eq!(two, contract.sender(alice).balance_of(alice));
        assert_eq!(two, contract.sender(alice).total_supply());

        contract.sender(bob).burn_from(alice, one).motsu_unwrap();

        assert_eq!(one, contract.sender(alice).balance_of(alice));
        assert_eq!(one, contract.sender(alice).total_supply());
        assert_eq!(U256::ZERO, contract.sender(alice).allowance(bob, alice));
    }

    #[motsu::test]
    fn burns_from_errors_when_insufficient_balance(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        // Alice approves `msg::sender`.

        let one = U256::ONE;
        contract.sender(alice).approve(bob, one).motsu_unwrap();
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(bob));

        let result = contract.sender(bob).burn_from(alice, one);
        assert!(matches!(
            result,
            Err(
                Error::InsufficientBalance(ERC20InsufficientBalance { sender, balance, needed })
            ) if sender == alice && balance.is_zero() && needed == one
        ));
    }

    #[motsu::test]
    fn burns_from_errors_when_insufficient_allowance(
        contract: Contract<Erc20>,
        alice: Address,
    ) {
        // Mint some tokens for Alice.
        let one = U256::ONE;
        contract
            .sender(alice)
            ._update(Address::ZERO, alice, one)
            .motsu_unwrap();
        assert_eq!(one, contract.sender(alice).balance_of(alice));

        let result = contract.sender(alice).burn_from(alice, one);
        assert!(matches!(
            result,
            Err(
                Error::InsufficientAllowance(ERC20InsufficientAllowance {
                    spender,
                    allowance,
                    needed,
                }))
                if spender == alice && allowance.is_zero() && needed == one,

        ));
    }
}
