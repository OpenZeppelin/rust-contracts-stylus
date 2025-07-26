//! Optional Burnable extension of the ERC-20 standard.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::token::erc20::{Erc20, IErc20};

/// Extension of [`Erc20`] that allows token holders to destroy both
/// their own tokens and those that they have an allowance for,
/// in a way that can be recognized off-chain (via event analysis).
pub trait IErc20Burnable: IErc20 {
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
    fn burn(&mut self, value: U256) -> Result<(), Vec<u8>> {
        self._burn(msg::sender(), value)
    }

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
    ) -> Result<(), Vec<u8>> {
        self._spend_allowance(account, msg::sender(), value)?;
        self._burn(account, value)
    }
}

impl IErc20Burnable for Erc20 {}

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::*;
    use stylus_sdk::call::MethodError;

    use super::IErc20Burnable;
    use crate::token::erc20::{
        ERC20InsufficientAllowance, ERC20InsufficientBalance, Erc20,
        Erc20Internal, Error, IErc20,
    };

    #[motsu::test]
    fn burns(contract: Contract<Erc20>, alice: Address) {
        let zero = U256::ZERO;
        let one = uint!(1_U256);

        assert_eq!(zero, contract.sender(alice).total_supply());

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
        let zero = U256::ZERO;
        let one = uint!(1_U256);

        assert_eq!(zero, contract.sender(alice).balance_of(alice));

        let result = contract.sender(alice).burn(one);
        assert_eq!(
            result,
            Err(Error::InsufficientBalance(ERC20InsufficientBalance {
                balance: zero,
                sender: alice,
                needed: one
            })
            .encode())
        );
    }

    #[motsu::test]
    fn burn_from(contract: Contract<Erc20>, alice: Address, bob: Address) {
        // Alice approves `msg::sender`.
        let one = uint!(1_U256);
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
        let zero = U256::ZERO;
        let one = uint!(1_U256);

        contract.sender(alice).approve(bob, one).motsu_unwrap();
        assert_eq!(zero, contract.sender(alice).balance_of(bob));

        let result = contract.sender(bob).burn_from(alice, one);
        assert_eq!(
            result,
            Err(Error::InsufficientBalance(ERC20InsufficientBalance {
                balance: zero,
                needed: one,
                sender: alice
            })
            .encode())
        );
    }

    #[motsu::test]
    fn burns_from_errors_when_insufficient_allowance(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        // Mint some tokens for Alice.
        let one = uint!(1_U256);
        contract
            .sender(alice)
            ._update(Address::ZERO, alice, one)
            .motsu_unwrap();
        assert_eq!(one, contract.sender(alice).balance_of(alice));

        let result = contract.sender(bob).burn_from(alice, one);
        assert_eq!(
            result,
            Err(Error::InsufficientAllowance(ERC20InsufficientAllowance {
                allowance: U256::ZERO,
                needed: one,
                spender: bob,
            })
            .encode())
        );

        // Verify one needs to give oneself allowance to invoke `burn_from`
        let result = contract.sender(alice).burn_from(alice, one);
        assert_eq!(
            result,
            Err(Error::InsufficientAllowance(ERC20InsufficientAllowance {
                allowance: U256::ZERO,
                needed: one,
                spender: alice,
            })
            .encode())
        );
    }
}
