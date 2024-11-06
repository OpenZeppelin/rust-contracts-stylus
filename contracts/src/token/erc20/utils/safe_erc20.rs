//! Wrappers around ERC-20 operations that throw on failure (when the token
//! contract returns false). Tokens that return no value (and instead revert or
//! throw on failure) are also supported, non-reverting calls are assumed to be
//! successful.
//! To use this library you can add a `#[inherit(SafeErc20)]` attribute to
//! your contract, which allows you to call the safe operations as
//! `contract.safe_transfer(token_addr, ...)`, etc.

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};
use stylus_sdk::{
    call::RawCall,
    contract::address,
    evm::gas_left,
    function_selector,
    storage::TopLevelStorage,
    stylus_proc::{public, sol_storage, SolidityError},
    types::AddressVM,
};

use crate::token::erc20;

sol! {
    /// An operation with an ERC-20 token failed.
    ///
    /// * `token` - Address of the ERC-20 token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error SafeErc20FailedOperation(address token);

    /// Indicates a failed [`ISafeErc20::decrease_allowance`] request.
    ///
    /// * `spender` - Address of future tokens' spender.
    /// * `current_allowance` - Current allowance of the `spender`.
    /// * `requested_decrease` - Requested decrease in allowance for `spender`.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error SafeErc20FailedDecreaseAllowance(
        address spender,
        uint256 current_allowance,
        uint256 requested_decrease
    );
}

/// A [`SafeErc20`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`erc20::Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(SafeErc20FailedOperation),
    /// Indicates a failed [`ISafeErc20::decrease_allowance`] request.
    SafeErc20FailedDecreaseAllowance(SafeErc20FailedDecreaseAllowance),
}

pub use token::*;
#[allow(missing_docs)]
mod token {
    alloy_sol_types::sol! {
        /// Interface of the ERC-20 token.
        interface IErc20 {
            function allowance(address owner, address spender) external view returns (uint256);
            function approve(address spender, uint256 value) external returns (bool);
            function transfer(address to, uint256 value) external returns (bool);
            function transferFrom(address from, address to, uint256 value) external returns (bool);
        }
    }
}
sol_storage! {
    /// State of the [`SafeErc20`] Contract.
    pub struct SafeErc20 {}
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for SafeErc20 {}

/// Required interface of an [`SafeErc20`] utility contract.
pub trait ISafeErc20 {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Transfer `value` amount of `token` from the calling contract to `to`. If
    /// `token` returns no value, non-reverting calls are assumed to be
    /// successful.
    fn safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Transfer `value` amount of `token` from `from` to `to`, spending the
    /// approval given by `from` to the calling contract. If `token` returns
    /// no value, non-reverting calls are assumed to be successful.
    fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Increase the calling contract's allowance toward `spender` by `value`.
    /// If `token` returns no value, non-reverting calls are assumed to be
    /// successful.
    ///
    /// # Panics
    ///
    /// If increased allowance exceeds `U256::MAX`.
    fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Decrease the calling contract's allowance toward `spender` by
    /// `requested_decrease`. If `token` returns no value, non-reverting
    /// calls are assumed to be successful.
    fn safe_decrease_allowance(
        &mut self,
        token: Address,
        spender: Address,
        requested_decrease: U256,
    ) -> Result<(), Self::Error>;

    /// Set the calling contract's allowance toward `spender` to `value`. If
    /// `token` returns no value, non-reverting calls are assumed to be
    /// successful. Meant to be used with tokens that require the approval
    /// to be set to zero before setting it to a non-zero value, such as USDT.
    fn force_approve(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error>;
}

#[public]
impl ISafeErc20 for SafeErc20 {
    type Error = Error;

    fn safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let call = IErc20::transferCall { to, value };

        self.call_optional_return(token, &call)
    }

    fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let call = IErc20::transferFromCall { from, to, value };

        self.call_optional_return(token, &call)
    }

    fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let current_allowance = self.allowance(token, spender)?;
        let new_allowance = current_allowance
            .checked_add(value)
            .expect("should not exceed `U256::MAX` for allowance");
        self.force_approve(token, spender, new_allowance)
    }

    fn safe_decrease_allowance(
        &mut self,
        token: Address,
        spender: Address,
        requested_decrease: U256,
    ) -> Result<(), Self::Error> {
        let current_allowance = self.allowance(token, spender)?;

        if current_allowance < requested_decrease {
            return Err(SafeErc20FailedDecreaseAllowance {
                spender,
                current_allowance,
                requested_decrease,
            }
            .into());
        }

        self.force_approve(
            token,
            spender,
            current_allowance - requested_decrease,
        )
    }

    fn force_approve(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let approve_call = IErc20::approveCall { spender, value };

        // Try performing the approval with the desired value
        if self.call_optional_return(token, &approve_call).is_ok() {
            return Ok(());
        }

        // If that fails, reset allowance to zero, then retry the desired
        // approval
        let reset_approval_call =
            IErc20::approveCall { spender, value: U256::ZERO };
        self.call_optional_return(token, &reset_approval_call)?;
        self.call_optional_return(token, &approve_call)?;

        Ok(())
    }
}

impl SafeErc20 {
    /// Imitates a Stylus high-level call, relaxing the requirement on the
    /// return value: if data is returned, it must not be `false`, otherwise
    /// calls are assumed to be successful.
    fn call_optional_return(
        &self,
        token: Address,
        call: &impl SolCall,
    ) -> Result<(), Error> {
        if !Address::has_code(&token) {
            return Err(SafeErc20FailedOperation { token }.into());
        }

        match RawCall::new()
            .gas(gas_left())
            .limit_return_data(0, 32)
            .call(token, &call.abi_encode())
        {
            Ok(data) if data.is_empty() || encodes_true(&data) => Ok(()),
            _ => Err(SafeErc20FailedOperation { token }.into()),
        }
    }

    fn allowance(
        &mut self,
        token: Address,
        spender: Address,
    ) -> Result<U256, Error> {
        if !Address::has_code(&token) {
            return Err(SafeErc20FailedOperation { token }.into());
        }

        let call = IErc20::allowanceCall { owner: address(), spender };
        let allowance = RawCall::new()
            .gas(gas_left())
            .limit_return_data(0, 32)
            .call(token, &call.abi_encode())
            .map_err(|_| {
                Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                    token,
                })
            })?;

        Ok(U256::from_be_slice(&allowance))
    }
}

fn encodes_true(data: &[u8]) -> bool {
    data.split_last().map_or(false, |(last, rest)| {
        *last == 1 && rest.iter().all(|&byte| byte == 0)
    })
}
