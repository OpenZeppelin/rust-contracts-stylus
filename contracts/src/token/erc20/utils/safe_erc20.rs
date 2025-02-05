//! Wrappers around ERC-20 operations that throw on failure (when the token
//! contract returns false).
//!
//! Tokens that return no value (and instead revert or
//! throw on failure) are also supported, non-reverting calls are assumed to be
//! successful.
//!
//! To use this library, you can add a `#[inherit(SafeErc20)]` attribute to
//! your contract, which allows you to call the safe operations as
//! `contract.safe_transfer(token_addr, ...)`, etc.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
pub use sol::*;
use stylus_sdk::{
    call::{MethodError, RawCall},
    contract::address,
    function_selector,
    prelude::storage,
    storage::TopLevelStorage,
    stylus_proc::{public, SolidityError},
    types::AddressVM,
};

use crate::{token::erc20, utils::ReentrantCallHandler};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// An operation with an ERC-20 token failed.
        ///
        /// * `token` - Address of the ERC-20 token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error SafeErc20FailedOperation(address token);

        /// Indicates a failed [`ISafeErc20::safe_decrease_allowance`] request.
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
}

/// A [`SafeErc20`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`erc20::Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(SafeErc20FailedOperation),
    /// Indicates a failed [`ISafeErc20::safe_decrease_allowance`] request.
    SafeErc20FailedDecreaseAllowance(SafeErc20FailedDecreaseAllowance),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

pub use token::*;
mod token {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
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

/// State of a [`SafeErc20`] Contract.
#[storage]
pub struct SafeErc20 {}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for SafeErc20 {}

/// Required interface of a [`SafeErc20`] utility contract.
pub trait ISafeErc20 {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Transfer `value` amount of `token` from the calling contract to `to`. If
    /// `token` returns no value, non-reverting calls are assumed to be
    /// successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    ///  * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///    contract , the contract fails to execute the call or the call returns
    ///    value that is not `true`.
    fn safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Transfer `value` amount of `token` from `from` to `to`, spending the
    /// approval given by `from` to the calling contract. If `token` returns
    /// no value, non-reverting calls are assumed to be successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    ///  * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///    contract , the contract fails to execute the call or the call returns
    ///    value that is not `true`.
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
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - Value to increase current allowance for `spender`.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///   contract, the contract fails to execute the call or the call returns
    ///   value that is not `true`.
    ///
    /// # Panics
    ///
    /// * If increased allowance exceeds `U256::MAX`.
    fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Decrease the calling contract's allowance toward `spender` by
    /// `requested_decrease`. If `token` returns no value, non-reverting
    /// calls are assumed to be successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `spender` - Account that will spend the tokens.
    /// * `requested_decrease` - Value allowed to be spent by `spender`.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///   contract, the contract fails to execute the call or the call returns
    ///   value that is not `true`.
    /// * [`Error::SafeErc20FailedDecreaseAllowance`] - If the current allowance
    ///   is less than `requested_decrease`.
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
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - Value allowed to be spent by `spender`.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///   contract, the contract fails to execute the call or the call returns
    ///   value that is not `true`.
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

        Self::call_optional_return(token, &call)
    }

    fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let call = IErc20::transferFromCall { from, to, value };

        Self::call_optional_return(token, &call)
    }

    fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let current_allowance = Self::allowance(token, spender)?;
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
        let current_allowance = Self::allowance(token, spender)?;

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

        // Try performing the approval with the desired value.
        if Self::call_optional_return(token, &approve_call).is_ok() {
            return Ok(());
        }

        // If that fails, reset the allowance to zero, then retry the desired
        // approval.
        let reset_approval_call =
            IErc20::approveCall { spender, value: U256::ZERO };
        Self::call_optional_return(token, &reset_approval_call)?;
        Self::call_optional_return(token, &approve_call)
    }
}

impl SafeErc20 {
    /// Imitates a Stylus high-level call, relaxing the requirement on the
    /// return value: if data is returned, it must not be `false`, otherwise
    /// calls are assumed to be successful.
    ///
    /// # Arguments
    ///
    /// * `token` - Address of the ERC-20 token contract.
    /// * `call` - [`IErc20`] call that implements [`SolCall`] trait.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///   contract, the contract fails to execute the call or the call returns
    ///   value that is not `true`.
    fn call_optional_return(
        token: Address,
        call: &impl SolCall,
    ) -> Result<(), Error> {
        if !Address::has_code(&token) {
            return Err(SafeErc20FailedOperation { token }.into());
        }

        match RawCall::new()
            .limit_return_data(0, 32)
            .call_with_reentrant_handling(token, &call.abi_encode())
        {
            Ok(data) if data.is_empty() || Self::encodes_true(&data) => Ok(()),
            _ => Err(SafeErc20FailedOperation { token }.into()),
        }
    }

    /// Returns the remaining number of ERC-20 tokens that `spender`
    /// will be allowed to spend on behalf of an owner.
    ///
    /// # Arguments
    ///
    /// * `token` - Address of the ERC-20 token contract.
    /// * `spender` - Account that will spend the tokens.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///   contract.
    /// * [`Error::SafeErc20FailedOperation`] - If the contract fails to read
    ///   `spender`'s allowance.
    fn allowance(token: Address, spender: Address) -> Result<U256, Error> {
        if !Address::has_code(&token) {
            return Err(SafeErc20FailedOperation { token }.into());
        }

        let call = IErc20::allowanceCall { owner: address(), spender };
        let result = RawCall::new()
            .limit_return_data(0, 32)
            .call_with_reentrant_handling(token, &call.abi_encode())
            .map_err(|_| {
                Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                    token,
                })
            })?;

        Ok(U256::from_be_slice(&result))
    }

    /// Returns true if a slice of bytes is an ABI encoded `true` value.
    ///
    /// # Arguments
    ///
    /// * `data` - Slice of bytes.
    fn encodes_true(data: &[u8]) -> bool {
        data.split_last().is_some_and(|(last, rest)| {
            *last == 1 && rest.iter().all(|&byte| byte == 0)
        })
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::SafeErc20;
    #[test]
    fn encodes_true_empty_slice() {
        assert!(!SafeErc20::encodes_true(&[]));
    }

    #[test]
    fn encodes_false_single_byte() {
        assert!(!SafeErc20::encodes_true(&[0]));
    }

    #[test]
    fn encodes_true_single_byte() {
        assert!(SafeErc20::encodes_true(&[1]));
    }

    #[test]
    fn encodes_false_many_bytes() {
        assert!(!SafeErc20::encodes_true(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
    }

    #[test]
    fn encodes_true_many_bytes() {
        assert!(SafeErc20::encodes_true(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]));
    }

    #[test]
    fn encodes_true_wrong_bytes() {
        assert!(!SafeErc20::encodes_true(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1]));
    }
}
