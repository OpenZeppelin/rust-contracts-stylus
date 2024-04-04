use stylus_sdk::{
    alloy_primitives::{Address, U256},
    msg,
    prelude::*,
};

use crate::erc20::{Error, ERC20};

sol_storage! {
    pub struct ERC20Burnable {
        ERC20 erc20;
    }
}

#[external]
#[inherit(ERC20)]
impl ERC20Burnable {
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
    pub fn burn(&mut self, value: U256) -> Result<(), Error> {
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
    pub fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Error> {
        self.erc20._spend_allowance(account, msg::sender(), value)?;
        self.erc20._burn(account, value)
    }
}

#[cfg(test)]
mod tests {}
