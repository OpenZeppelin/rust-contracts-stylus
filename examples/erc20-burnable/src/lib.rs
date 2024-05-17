#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

// That is example implementation of ERC20 with Burnable flavour.
// Extension of ERC20 that allows token holders to destroy both their own
// tokens and those that they have an allowance for, in a way that can be
// recognized off-chain (via event analysis).

use contracts::erc20::ERC20;
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC20 erc20;
    }
}

#[external]
#[inherit(ERC20)]
impl Token {
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
    pub(crate) fn burn(
        &mut self,
        value: alloy_primitives::U256,
    ) -> Result<(), alloc::vec::Vec<u8>> {
        self.erc20._burn(stylus_sdk::msg::sender(), value).map_err(|e| e.into())
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
        account: alloy_primitives::Address,
        value: alloy_primitives::U256,
    ) -> Result<(), alloc::vec::Vec<u8>> {
        self.erc20._spend_allowance(
            account,
            stylus_sdk::msg::sender(),
            value,
        )?;
        self.erc20._burn(account, value).map_err(|e| e.into())
    }
}
