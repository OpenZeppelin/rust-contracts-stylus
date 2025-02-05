//! Extension of the ERC-20 token contract to support token wrapping.
//!
//! Users can deposit and withdraw "underlying tokens" and receive a matching
//! number of "wrapped tokens". This is useful in conjunction with other
//! modules.
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
    call::Call,
    contract, msg,
    prelude::storage,
    storage::{StorageAddress, TopLevelStorage},
    stylus_proc::SolidityError,
};

use crate::token::erc20::{
    self,
    utils::{safe_erc20, IErc20 as IErc20Solidity, ISafeErc20, SafeErc20},
    Erc20, IErc20,
};

sol! {
    /// Indicates that he address is not a valid ERC-20 token.
    ///
    /// * `address` - Address of the invalid underling ERC-20 token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidUnderlying(address token);

    /// Indicates that the address is not an Invalid Sender address.
    ///
    /// * `sender` - Address  is an invalid sender.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC20InvalidSender(address sender);

    /// Indicates that The address is not a valid Invalid Asset.
    ///
    /// * `asset` - Address of the invalid  address of the token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error InvalidAsset(address asset);

    /// Indicates thata the address is not an invalid receiver addresss.
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

    /// The address is not a valid ERC-20 token.
    InvalidAsset(InvalidAsset),

    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
}
/// State of an [`Erc20Wrapper`] token.
#[storage]
pub struct Erc20Wrapper {
    /// Token Address of the  underline token
    #[allow(clippy::used_underscore_binding)]
    pub(crate) underlying_address: StorageAddress,

    /// [`SafeErc20`] contract
    safe_erc20: SafeErc20,
}

/// ERC-20 Wrapper Standard Interface
pub trait IErc20Wrapper {
    /// The error type associated to the` trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the address of the underlying token that is been wrapped.
    fn underlying(&self) -> Address;

    /// Allow a user to deposit underlying tokens and mint the corresponding
    /// number of wrapped token
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to deposit tokens to.
    /// * `value` - The amount of tokens to deposit.
    ///  
    /// # Errors
    ///
    /// * If the sender address is `contract:address()` or invalid,
    ///   [`Error::InvalidSender`] is returned.
    /// * If the receiver address is `contract:address()` or invalid,
    ///   [`Error::InvalidReceiver`] is returned.
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
    /// * `account` - The account to withdraw tokens to.
    /// * `value` - The amount of tokens to withdraw.
    /// * `erc20` - A mutable reference to the Erc20 contract.
    ///
    /// # Errors
    ///
    /// * If the receiver address is `contract:address()` or invalid,
    ///   [`Error::InvalidReceiver`] is returned.
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

    fn underlying(&self) -> Address {
        self.underlying_address.get()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
        erc20: &mut Erc20,
    ) -> Result<bool, Error> {
        let underlined_token = self.underlying_address.get();
        let sender = msg::sender();

        if sender == contract::address() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: account,
            }));
        }

        if account == contract::address() {
            return Err(Error::InvalidSender(ERC20InvalidSender {
                sender: contract::address(),
            }));
        }

        self.safe_erc20.safe_transfer_from(
            self.underlying(),
            sender,
            contract::address(),
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
    ) -> Result<bool, Error> {
        let underlined_token = self.underlying_address.get();
        if account == contract::address() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: account,
            }));
        }
        erc20._burn(account, value)?;
        self.safe_erc20.safe_transfer(self.underlying(), account, value)?;
        Ok(true)
    }
}

impl Erc20Wrapper {
    /// Mints wrapped tokens to cover any underlying tokens that might have been
    /// mistakenly transferred or acquired through rebasing mechanisms.
    ///
    /// This is an internal function that can be exposed with access control if
    /// required.
    ///
    /// Arguments:
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - The account to mint tokens to.
    /// * `erc20` - A mutable reference to the Erc20 contract.
    ///
    /// # Errors
    ///
    /// If the external call for balance of fails , then the error
    /// [`Error::InvalidAsset`] is returned.
    pub fn _recover(
        &mut self,
        account: Address,
        erc20: &mut Erc20,
    ) -> Result<U256, Error> {
        let underline_token = IErc20Solidity::new(self.underlying());
        let value = underline_token
            .balance_of(Call::new_in(self), contract::address())
            .map_err(|_| InvalidAsset { asset: contract::address() })?;
        erc20._mint(account, value)?;
        Ok(U256::from(value))
    }
}

// TODO: Add missing tests once `motsu` supports calling external contracts.
#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::address;
    use stylus_sdk::prelude::storage;

    use super::{Erc20Wrapper, IErc20Wrapper};

    #[storage]
    struct Erc20WrapperTestExample {
        wrapper: Erc20Wrapper,
    }

    #[motsu::test]
    fn underlying_works(contract: Erc20WrapperTestExample) {
        let asset = address!("DeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF");
        contract.wrapper.underlying_address.set(asset);
        assert_eq!(contract.wrapper.underlying(), asset);
    }
}
