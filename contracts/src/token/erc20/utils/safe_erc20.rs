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

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::SolCall;
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::{MethodError, RawCall},
    contract::address,
    function_selector,
    prelude::*,
    types::AddressVM,
};

use crate::{
    token::erc20,
    utils::introspection::erc165::{Erc165, IErc165},
};

const BOOL_TYPE_SIZE: usize = 32;

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

        /// Interface of the ERC-1363 token.
        interface IErc1363 {
            function transferAndCall(address to, uint256 value, bytes data) external returns (bool);
            function transferFromAndCall(address from, address to, uint256 value, bytes data) external returns (bool);
            function approveAndCall(address spender, uint256 value, bytes data) external returns (bool);
        }
    }
}

/// State of a [`SafeErc20`] Contract.
#[storage]
pub struct SafeErc20;

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for SafeErc20 {}

/// Required interface of a [`SafeErc20`] utility contract.
#[interface_id]
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

    /// Variant of `safe_transfer` that returns a bool instead of reverting if the operation is not successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the transfer was successful
    /// * `Ok(false)` if the transfer failed
    /// * `Err(_)` if there was an error checking the token contract
    fn try_safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error>;

    /// Variant of `safe_transfer_from` that returns a bool instead of reverting if the operation is not successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the transfer was successful
    /// * `Ok(false)` if the transfer failed
    /// * `Err(_)` if there was an error checking the token contract
    fn try_safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error>;

    /// Increase the calling contract's allowance toward `spender` by `value`. If `token` returns no value,
    /// non-reverting calls are assumed to be successful.
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

    /// Performs an ERC1363 transferAndCall, with a fallback to the simple ERC20 transfer if the target has no
    /// code. This can be used to implement an ERC721-like safe transfer that rely on ERC1363 checks when
    /// targeting contracts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-1363 token contract.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    /// * `data` - Additional data to be passed to the receiver contract.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the transfer fails.
    fn transfer_and_call_relaxed(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Self::Error>;

    /// Performs an ERC1363 transferFromAndCall, with a fallback to the simple ERC20 transferFrom if the target
    /// has no code. This can be used to implement an ERC721-like safe transfer that rely on ERC1363 checks when
    /// targeting contracts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-1363 token contract.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    /// * `data` - Additional data to be passed to the receiver contract.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the transfer fails.
    fn transfer_from_and_call_relaxed(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Self::Error>;

    /// Performs an ERC1363 approveAndCall, with a fallback to the simple ERC20 approve if the target has no
    /// code. This can be used to implement an ERC721-like safe transfer that rely on ERC1363 checks when
    /// targeting contracts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-1363 token contract.
    /// * `to` - Account to approve tokens for.
    /// * `value` - Number of tokens to approve.
    /// * `data` - Additional data to be passed to the receiver contract.
    ///
    /// # Errors
    ///
    /// * [`Error::SafeErc20FailedOperation`] - If the approval fails.
    fn approve_and_call_relaxed(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
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
        Self::call_optional_return(&token, &call)
    }

    fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let call = IErc20::transferFromCall { from, to, value };
        Self::call_optional_return(&token, &call)
    }

    fn try_safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        let call = IErc20::transferCall { to, value };
        Self::call_optional_return_bool(&token, &call)
    }

    fn try_safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        let call = IErc20::transferFromCall { from, to, value };
        Self::call_optional_return_bool(&token, &call)
    }

    fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let old_allowance = Self::allowance(&token, spender)?;
        Self::force_approve(&token, spender, old_allowance.checked_add(value).ok_or(Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }))?)
    }

    fn safe_decrease_allowance(
        &mut self,
        token: Address,
        spender: Address,
        requested_decrease: U256,
    ) -> Result<(), Self::Error> {
        let current_allowance = Self::allowance(&token, spender)?;
        if current_allowance < requested_decrease {
            return Err(Error::SafeErc20FailedDecreaseAllowance(
                SafeErc20FailedDecreaseAllowance {
                    spender,
                    current_allowance,
                    requested_decrease,
                },
            ));
        }
        Self::force_approve(&token, spender, current_allowance - requested_decrease)
    }

    fn force_approve(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let approve_call = IErc20::approveCall { spender, value };
        
        // Try direct approve first
        if Self::call_optional_return_bool(&token, &approve_call)? {
            return Ok(());
        }

        // If it failed, fallback to zero-reset strategy
        let reset_call = IErc20::approveCall {
            spender,
            value: U256::from(0),
        };
        Self::call_optional_return(&token, &reset_call)?;
        Self::call_optional_return(&token, &approve_call)
    }

    fn transfer_and_call_relaxed(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Self::Error> {
        if Self::account_has_code(to) == 0 {
            self.safe_transfer(token, to, value)
        } else {
            let call = IErc1363::transferAndCallCall { to, value, data };
            if !Self::call_optional_return_bool(&token, &call)? {
                return Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }));
            }
            Ok(())
        }
    }

    fn transfer_from_and_call_relaxed(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Self::Error> {
        if Self::account_has_code(to) == 0 {
            self.safe_transfer_from(token, from, to, value)
        } else {
            let call = IErc1363::transferFromAndCallCall { from, to, value, data };
            if !Self::call_optional_return_bool(&token, &call)? {
                return Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }));
            }
            Ok(())
        }
    }

    fn approve_and_call_relaxed(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Self::Error> {
        if Self::account_has_code(to) == 0 {
            self.force_approve(token, to, value)
        } else {
            let call = IErc1363::approveAndCallCall { spender: to, value, data };
            if !Self::call_optional_return_bool(&token, &call)? {
                return Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }));
            }
            Ok(())
        }
    }
}

impl SafeErc20 {
    #[inline]
    fn account_has_code(addr: Address) -> usize {
        // SAFETY: extcodesize is a pure query, no state mutation or re-entrancy
        unsafe { RawCall::new().code_length(&addr) }
    }

    fn call_optional_return(
        token: &Address,
        call: &impl SolCall,
    ) -> Result<(), Error> {
        let mut return_data = vec![0u8; BOOL_TYPE_SIZE];
        let success = unsafe {
            RawCall::new()
                .gas(u64::MAX)
                .call(token, call.encode())
                .copy_into(&mut return_data)
        };
        if !success {
            return Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                token: *token,
            }));
        }
        // Treat no-data (all zeros) as success; only fail if there's non-zero junk that isn't `true`
        if return_data.iter().any(|&b| b != 0) && !Self::encodes_true(&return_data) {
            return Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                token: *token,
            }));
        }
        Ok(())
    }

    fn call_optional_return_bool(
        token: &Address,
        call: &impl SolCall,
    ) -> Result<bool, Error> {
        let mut return_data = vec![0u8; BOOL_TYPE_SIZE];
        let success = unsafe {
            RawCall::new()
                .gas(u64::MAX)
                .call(token, call.encode())
                .copy_into(&mut return_data)
        };

        if !success {
            return Ok(false);
        }

        Ok(Self::encodes_true(&return_data))
    }

    fn allowance(token: &Address, spender: Address) -> Result<U256, Error> {
        let call = IErc20::allowanceCall {
            owner: address(),
            spender,
        };
        let mut return_data = vec![0u8; 32];
        let success = unsafe {
            RawCall::new()
                .gas(u64::MAX)
                .call(token, call.encode())
                .copy_into(&mut return_data)
        };

        if !success {
            return Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                token: *token,
            }));
        }

        Ok(U256::from_be_bytes(return_data.try_into().unwrap()))
    }

    fn encodes_true(data: &[u8]) -> bool {
        if data.is_empty() {
            return true;
        }
        data.len() == 32
            && data[31] == 1
            && data[..31].iter().all(|&b| b == 0)
    }
}

impl IErc165 for SafeErc20 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as ISafeErc20>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::{ISafeErc20, SafeErc20};
    use crate::utils::introspection::erc165::IErc165;

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

    #[motsu::test]
    fn interface_id() {
        let actual = <SafeErc20 as ISafeErc20>::INTERFACE_ID;
        let expected = 0xf71993e3;
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface() {
        assert!(SafeErc20::supports_interface(
            <SafeErc20 as IErc165>::INTERFACE_ID.into()
        ));
        assert!(SafeErc20::supports_interface(
            <SafeErc20 as ISafeErc20>::INTERFACE_ID.into()
        ));

        let fake_interface_id = 0x12345678u32;
        assert!(!SafeErc20::supports_interface(fake_interface_id.into()));
    }
}
