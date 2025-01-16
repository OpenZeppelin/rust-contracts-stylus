//! Extension of the ERC-20 token contract to support token wrapping.
//!
//! Users can deposit and withdraw "underlying tokens" and receive a matching
//! number of "wrapped tokens". This is useful in conjunction with other
//! modules. For example, combining this wrapping mechanism with {ERC20Votes}
//! will allow the wrapping of an existing "basic" ERC-20 into a governance
//! token.
//!
//! WARNING: Any mechanism in which the underlying token changes the {balanceOf}
//! of an account without an explicit transfer may desynchronize this contract's
//! supply and its underlying balance. Please exercise caution when wrapping
//! tokens that may undercollateralize the wrapper (i.e. wrapper's total supply
//! is higher than its underlying balance). See {_recover} for recovering value
//! accrued to the wrapper.

use alloy_primitives::{Address, U256};
use alloy_sol_macro::sol;
use stylus_sdk::{
    contract, msg,
    prelude::storage,
    storage::{StorageAddress, TopLevelStorage},
    stylus_proc::SolidityError,
};

use super::IErc20Metadata;
use crate::token::erc20::{
    self,
    extensions::Erc20Metadata,
    utils::{
        safe_erc20::{self, ISafeErc20},
        SafeErc20,
    },
    Erc20,
};

sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidUnderlying(address token);

}

/// An [`Erc20Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),

    /// The underlying token couldn't be wrapped.
    InvalidUnderlying(ERC20InvalidUnderlying),
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}
/// State of an [`Erc4626`] token.
#[storage]
pub struct Erc20Wrapper {
    /// Token Address of the vault
    #[allow(clippy::used_underscore_binding)]
    pub _underlying: StorageAddress,
}

/// ERC-4626 Tokenized Vault Standard Interface
pub trait IERC20Wrapper {
    /// The error type associated to this ERC-4626 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    ///  Allow a user to deposit underlying tokens and mint the corresponding
    /// number of wrapped token
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn decimals(&self) -> Result<bool, Self::Error> {
    ///     self.token.decimals(account, &self.erc20)
    /// }
    /// ``
    fn decimals(&self, erc20: &mut Erc20Metadata) -> u8;

    /// Returns the address of the underlying token that is bben wrapped.
    fn underlying(&self) -> Address;

    ///  Allow a user to deposit underlying tokens and mint the corresponding
    /// number of wrapped token
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn deposit_to(&self,account:Address, value:U256) -> Result<bool, Self::Error> {
    ///     self.token.deposit_to(account,value, &self.erc20, &self.safe_erc20)
    /// }
    /// ```
    fn deposit_to(
        &self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
        safe_erc20: &mut SafeErc20,
    ) -> Result<bool, Self::Error>;

    /// Allow a user to burn a number of wrapped tokens and withdraw the
    /// corresponding number of underlying tokens.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// fn withdraw_to(&self,account:Address, value:U256) -> Result<bool, Self::Error> {
    ///     self.token.deposit_to(account,value, &self.erc20, &self.safe_erc20)
    /// }
    /// ```
    fn withdraw_to(
        &self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
        safe_erc20: &mut SafeErc20,
    ) -> Result<bool, Self::Error>;
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc20Wrapper {}

impl IERC20Wrapper for Erc20Wrapper {
    type Error = Error;

    fn underlying(&self) -> Address {
        self._underlying.get()
    }

    fn decimals(&self, erc20_metadata: &mut Erc20Metadata) -> u8 {
        erc20_metadata.decimals()
    }

    fn deposit_to(
        &self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
        safe_erc20: &mut SafeErc20,
    ) -> Result<bool, Error> {
        let underlined_token = self._underlying.get();
        let sender = msg::sender();
        if account == contract::address() {
            return Err(Error::InvalidUnderlying(ERC20InvalidUnderlying {
                token: contract::address(),
            }));
        }

        if sender == contract::address() {
            return Err(Error::InvalidUnderlying(ERC20InvalidUnderlying {
                token: account,
            }));
        }
        safe_erc20.safe_transfer_from(
            underlined_token,
            sender,
            contract::address(),
            value,
        )?;
        erc20._mint(account, value)?;
        Ok(true)
    }

    fn withdraw_to(
        &self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
        safe_erc20: &mut SafeErc20,
    ) -> Result<bool, Error> {
        let underlined_token = self._underlying.get();
        if account == contract::address() {
            return Err(Error::InvalidUnderlying(ERC20InvalidUnderlying {
                token: contract::address(),
            }));
        }
        erc20._burn(account, value)?;
        safe_erc20.safe_transfer(underlined_token, account, value)?;
        Ok(true)
    }
}

impl Erc20Wrapper {
    fn _recover(&self) -> Address {
        return contract::address();
    }
}
