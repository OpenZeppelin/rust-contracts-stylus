//! Wrappers around ERC-20 operations that throw on failure.

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolValue};
use stylus_proc::{public, sol_interface, sol_storage, SolidityError};
use stylus_sdk::{
    call::{Call, RawCall},
    contract::address,
    evm::gas_left,
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

sol_interface! {
    /// Interface of the ERC-20 standard as defined in the ERC.
    interface IERC20 {
        /// Returns the remaining number of tokens that `spender` will be
        /// allowed to spend on behalf of `owner` through {transferFrom}. This is
        /// zero by default.
        ///
        /// This value changes when {approve} or {transferFrom} are called.
        function allowance(address owner, address spender) external view returns (uint256);
    }
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
        let encoded_args = (to, value).abi_encode_params();
        let selector = function_selector!("transfer", Address, U256);
        // Combine function selector and input data (use abi_packed way)
        let data = [&selector[..4], &encoded_args].concat();

        self.call_optional_return(token, &data)
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
        let encoded_args = (from, to, value).abi_encode_params();
        let selector =
            function_selector!("transferFrom", Address, Address, U256);
        // Combine function selector and input data (use abi_packed way)
        let data = [&selector[..4], &encoded_args].concat();

        self.call_optional_return(token, &data)
    }

    /// Increase the calling contract's allowance toward `spender` by `value`. If `token` returns no value,
    /// non-reverting calls are assumed to be successful.
    pub fn safe_increase_allowance(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Error> {
        let erc20 = IERC20::new(token);
        let call = Call::new_in(self);
        let old_allowance = erc20.allowance(call, address(), spender).or(
            Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                token,
            })),
        )?;
        self.force_approve(token, spender, old_allowance + value)
    }

    /// Decrease the calling contract's allowance toward `spender` by `requestedDecrease`. If `token` returns no
    /// value, non-reverting calls are assumed to be successful.
    pub fn safe_decrease_allowance(
        &mut self,
        token: Address,
        spender: Address,
        requested_decrease: U256,
    ) -> Result<(), Error> {
        let erc20 = IERC20::new(token);
        let call = Call::new_in(self);
        let current_allowance =
            erc20.allowance(call, address(), spender).or({
                Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                    token,
                }))
            })?;

        if current_allowance < requested_decrease {
            return Err(Error::SafeErc20FailedDecreaseAllowance(
                SafeErc20FailedDecreaseAllowance {
                    spender,
                    currentAllowance: current_allowance,
                    requestedDecrease: requested_decrease,
                },
            ));
        }

        self.force_approve(
            token,
            spender,
            current_allowance - requested_decrease,
        )
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
        let selector = function_selector!("approve", Address, U256);

        // Helper function to construct calldata
        fn build_approve_calldata(
            spender: Address,
            value: U256,
            selector: &[u8],
        ) -> Vec<u8> {
            let encoded_args = (spender, value).abi_encode_params();
            [&selector[..4], &encoded_args].concat()
        }

        // Try performing the approval with the desired value
        let approve_data = build_approve_calldata(spender, value, &selector);
        if self.call_optional_return(token, &approve_data).is_ok() {
            return Ok(());
        }

        // If that fails, reset allowance to zero, then retry the desired approval
        let reset_data = build_approve_calldata(spender, U256::ZERO, &selector);
        self.call_optional_return(token, &reset_data)?;
        self.call_optional_return(token, &approve_data)?;

        Ok(())
    }
}

impl SafeErc20 {
    /// Imitates a Solidity high-level call, relaxing the requirement on the return value:
    /// if data is returned, it must not be `false`, otherwise calls are assumed to be successful.
    fn call_optional_return(
        &self,
        token: Address,
        data: &[u8],
    ) -> Result<(), Error> {
        match RawCall::new()
            .gas(gas_left())
            .limit_return_data(0, 32)
            .call(token, data)
        {
            Ok(data)
                if !(data.is_empty() && !Address::has_code(&token))
                    && encodes_true(&data) =>
            {
                Ok(())
            }
            _ => {
                Err(Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                    token,
                }))
            }
        }
    }
}

fn encodes_true(data: &[u8]) -> bool {
    data[..data.len() - 1].iter().all(|&byte| byte == 0)
        && data[data.len() - 1] == 1
}
