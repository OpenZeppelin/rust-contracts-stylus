//! Extension of the ERC-20 token contract to support token wrapping.
//!
//! Users can deposit and withdraw "underlying tokens" and receive a matching
//! number of "wrapped tokens". This is useful in conjunction with other
//! modules.
//!
//! WARNING: Any mechanism in which the underlying token changes the
//! [`IErc20::balance_of`] of an account without an explicit transfer may
//! desynchronize this contract's supply and its underlying balance. Please
//! exercise caution when wrapping tokens that may undercollateralize the
//! wrapper (i.e. wrapper's total supply is higher than its underlying balance).
//! See [`Erc20Wrapper::_recover`] for recovering value accrued to the wrapper.

use alloy_primitives::{Address, U256, U8};
use alloy_sol_macro::sol;
use stylus_sdk::{
    call::Call,
    contract, msg,
    prelude::*,
    storage::{StorageAddress, StorageU8, TopLevelStorage},
    stylus_proc::SolidityError,
};

use crate::token::erc20::{
    self,
    utils::{safe_erc20, IErc20 as IErc20Solidity, ISafeErc20, SafeErc20},
    Erc20, IErc20,
};

sol! {
    /// Indicates that the address is not a valid ERC-20 token.
    ///
    /// * `address` - Address of the invalid ERC-20 token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidUnderlying(address token);

    /// Indicates that the address is not a valid sender address.
    ///
    /// * `sender` - Address of the invalid sender.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidSender(address sender);

    /// Indicates that the address is not a valid receiver addresss.
    ///
    /// * `receiver` - Address of the invalid receiver.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidReceiver(address receiver);

}

/// An [`Erc20Wrapper`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`SafeErc20`] contract [`safe_erc20::Error`].
    SafeErc20(safe_erc20::Error),

    /// The Sender Address is not valid.
    InvalidSender(ERC20InvalidSender),

    /// The Reciver Address is not valid.
    InvalidReceiver(ERC20InvalidReceiver),

    /// The underlying token couldn't be wrapped.
    InvalidUnderlying(ERC20InvalidUnderlying),

    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}
/// State of an [`Erc20Wrapper`] token.
#[storage]
pub struct Erc20Wrapper {
    /// Token Address of the  underline token
    #[allow(clippy::used_underscore_binding)]
    pub(crate) underlying_address: StorageAddress,
    /// Token decimals.
    pub(crate) underlying_decimals: StorageU8,
    /// [`SafeErc20`] contract
    safe_erc20: SafeErc20,
}

/// ERC-20 Wrapper Standard Interface
pub trait IErc20Wrapper {
    /// The error type associated to the` trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the number of decimals used to get its user representation.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn decimals(&self) -> U8;

    /// Returns the address of the underlying ERC-20 token that is being
    /// wrapped.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn underlying(&self) -> Address;

    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// number of wrapped tokens.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to deposit tokens to.
    /// * `value` - The amount of tokens to deposit.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `msg::sender()`'s address is the
    ///   `contract:address()`.
    /// * [`Error::InvalidReceiver`] - If the `account` address is a
    ///   `contract:address()`.
    /// * [`Error::SafeErc20`] - If caller lacks sufficient balance or hasn't
    ///   approved enough tokens to the [`Erc20Wrapper`] contract.
    /// * [`Error::Erc20`] - If an error occurrs during [`Erc20::_mint`]
    ///   operation.
    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error>;

    /// Allow a user to burn a number of wrapped tokens and withdraw the
    /// corresponding number of underlying tokens.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to withdraw tokens from.
    /// * `value` - The amount of tokens to withdraw.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `account`'s address is a
    ///   `contract:address()`.
    /// * [`Error::Erc20`] - If an error occurrs during [`Erc20::_burn`]
    ///   operation.
    /// * [`Error::SafeErc20`] - If the [`Erc20Wrapper`] contract lacks
    ///   sufficient balance.
    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error>;
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc20Wrapper {}

impl IErc20Wrapper for Erc20Wrapper {
    type Error = Error;

    fn decimals(&self) -> U8 {
        self.underlying_decimals.get()
    }

    fn underlying(&self) -> Address {
        self.underlying_address.get()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error> {
        let contract_address = contract::address();
        let sender = msg::sender();

        if sender == contract_address {
            return Err(ERC20InvalidSender { sender }.into());
        }

        if account == contract_address {
            return Err(ERC20InvalidReceiver { receiver: account }.into());
        }

        self.safe_erc20.safe_transfer_from(
            self.underlying(),
            sender,
            contract_address,
            value,
        )?;

        erc20._mint(account, value)?;

        Ok(true)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Self::Error> {
        if account == contract::address() {
            return Err(ERC20InvalidReceiver { receiver: account }.into());
        }

        erc20._burn(msg::sender(), value)?;

        self.safe_erc20.safe_transfer(self.underlying(), account, value)?;

        Ok(true)
    }
}

impl Erc20Wrapper {
    /// Mint wrapped token to cover any underlying tokens that would have been
    /// transferred by mistake or acquired from rebasing mechanisms.
    ///
    /// Internal function that can be exposed with access control if desired.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `erc20` - Write access to an [`Erc20`] contract.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidUnderlying`]  - If the external call for
    ///   [`IErc20::balance_of`] fails.
    /// * [`Error::Erc20`] - If an error occurrs during [`Erc20::_mint`]
    ///   operation.
    pub fn _recover(
        &mut self,
        account: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let contract_address = contract::address();

        let underline_token = IErc20Solidity::new(self.underlying());

        let underlying_balance = underline_token
            .balance_of(Call::new_in(self), contract_address)
            .map_err(|_| ERC20InvalidUnderlying { token: contract_address })?;

        let value = underlying_balance - erc20.total_supply();

        erc20._mint(account, value)?;

        Ok(value)
    }
}

// TODO: Add missing tests once `motsu` supports calling external contracts.
#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::address;
    use motsu::prelude::Contract;
    use stylus_sdk::prelude::*;

    use super::*;

    #[storage]
    struct Erc20WrapperTestExample {
        wrapper: Erc20Wrapper,
        erc20: Erc20,
    }

    #[public]
    impl Erc20WrapperTestExample {
        fn underlying(&self) -> Address {
            self.wrapper.underlying()
        }

        fn deposit_for(
            &mut self,
            account: Address,
            value: U256,
        ) -> Result<bool, Error> {
            self.wrapper.deposit_for(account, value, &mut self.erc20)
        }

        fn withdraw_to(
            &mut self,
            account: Address,
            value: U256,
        ) -> Result<bool, Error> {
            self.wrapper.withdraw_to(account, value, &mut self.erc20)
        }
    }

    unsafe impl TopLevelStorage for Erc20WrapperTestExample {}
    #[motsu::test]
    fn underlying_works(
        contract: Contract<Erc20WrapperTestExample>,
        alice: Address,
    ) {
        let asset = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");
        contract.init(alice, |contract| {
            contract.wrapper.underlying_address.set(asset);
        });
        assert_eq!(contract.sender(alice).wrapper.underlying(), asset);
    }
}
