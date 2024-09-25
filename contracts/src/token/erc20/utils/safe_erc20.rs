//! Wrappers around ERC-20 operations that throw on failure.

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use alloy_sol_types::{
    sol,
    sol_data::{Address as SOLAddress, Uint},
    SolType,
};
use stylus_proc::{public, sol_interface, sol_storage, SolidityError};
use stylus_sdk::{
    call::{call, Call},
    function_selector,
    storage::TopLevelStorage,
    types::AddressVM,
};

use crate::token::erc20;

sol! {
    /// An operation with an ERC-20 token failed.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error SafeErc20FailedOperation(address token);

     /// Indicates a failed `decreaseAllowance` request.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error SafeErc20FailedDecreaseAllowance(address spender, uint256 currentAllowance, uint256 requestedDecrease);
}

/// A SafeErc20 error
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc20`] contract [`erc20::Error`].
    Erc20(erc20::Error),
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(SafeErc20FailedOperation),
    /// Indicates a failed `decreaseAllowance` request.
    SafeErc20FailedDecreaseAllowance(SafeErc20FailedDecreaseAllowance),
}

sol_storage! {
    /// Wrappers around ERC-20 operations that throw on failure (when the token
    /// contract returns false). Tokens that return no value (and instead revert or
    /// throw on failure) are also supported, non-reverting calls are assumed to be
    /// successful.
    /// To use this library you can add a `using SafeERC20 for IERC20;` statement to
    /// your contract, which allows you to call the safe operations as
    /// `token.safeTransfer(...)`, etc.
    pub struct SafeErc20 {}
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for SafeErc20 {}

#[public]
impl SafeErc20 {
    /// Transfer `value` amount of `token` from the calling contract to `to`. If
    /// `token` returns no value, non-reverting calls are assumed to be
    /// successful.
    pub fn safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        type TransferType = (SOLAddress, Uint<256>);
        let tx_data = (to, value);
        let data = TransferType::abi_encode_params(&tx_data);
        let hashed_function_selector =
            function_selector!("transfer", Address, U256);
        // Combine function selector and input data (use abi_packed way)
        let calldata = [&hashed_function_selector[..4], &data].concat();

        self.call_optional_return(token, calldata)
    }

    /// Transfer `value` amount of `token` from `from` to `to`, spending the approval given by `from` to the
    /// calling contract. If `token` returns no value, non-reverting calls are assumed to be successful.
    pub fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        type TransferType = (SOLAddress, SOLAddress, Uint<256>);
        let tx_data = (from, to, value);
        let data = TransferType::abi_encode_params(&tx_data);
        let hashed_function_selector =
            function_selector!("transferFrom", Address, Address, U256);
        // Combine function selector and input data (use abi_packed way)
        let calldata = [&hashed_function_selector[..4], &data].concat();

        self.call_optional_return(token, calldata)
    }

    /// Increase the calling contract's allowance toward `spender` by `value`. If `token` returns no value,
    /// non-reverting calls are assumed to be successful.
    pub fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Error> {
        todo!()
    }

    /// Decrease the calling contract's allowance toward `spender` by `requestedDecrease`. If `token` returns no
    /// value, non-reverting calls are assumed to be successful.
    pub fn safe_decrease_allowance(
        &mut self,
        token: Address,
        spender: Address,
        requested_decrease: U256,
    ) -> Result<(), Error> {
        todo!()
    }

    /// Set the calling contract's allowance toward `spender` to `value`. If `token` returns no value,
    /// non-reverting calls are assumed to be successful. Meant to be used with tokens that require the approval
    /// to be set to zero before setting it to a non-zero value, such as USDT.
    pub fn force_approve(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Error> {
        type TransferType = (SOLAddress, Uint<256>);
        let tx_data = (spender, value);
        let data = TransferType::abi_encode_params(&tx_data);
        let hashed_function_selector =
            function_selector!("approve", Address, U256);
        // Combine function selector and input data (use abi_packed way)
        let approve_calldata = [&hashed_function_selector[..4], &data].concat();

        if self.call_optional_return(token, approve_calldata.clone()).is_err() {
            let tx_data = (spender, U256::ZERO);
            let data = TransferType::abi_encode_params(&tx_data);
            self.call_optional_return(
                token,
                [&hashed_function_selector[..4], &data].concat(),
            )?;
            self.call_optional_return(token, approve_calldata)?;
        }

        Ok(())
    }
}

impl SafeErc20 {
    /// Imitates a Solidity high-level call (i.e. a regular function call to a
    /// contract), relaxing the requirement on the return value: the return
    /// value is optional (but if data is returned, it must not be false).
    /// @param token The token targeted by the call.
    /// @param data The call data (encoded using abi.encode or one of its
    /// variants).
    ///
    /// This is a variant of {_callOptionalReturnBool} that reverts if call
    /// fails to meet the requirements.
    fn call_optional_return(
        &self,
        token: Address,
        data: Vec<u8>,
    ) -> Result<(), Error> {
        match call(Call::new(), token, data.as_slice()) {
            Ok(data) => {
                if data.is_empty() && !Address::has_code(&token) {
                    return Err(Error::SafeErc20FailedOperation(
                        SafeErc20FailedOperation { token },
                    ));
                }
            }
            Err(_) => {
                return Err(Error::SafeErc20FailedOperation(
                    SafeErc20FailedOperation { token },
                ))
            }
        }
        Ok(())
    }
}
