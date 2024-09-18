//! Wrappers around ERC-20 operations that throw on failure.

use crate::token::{erc20, erc20::Erc20};
use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use alloy_sol_types::{
    sol,
    sol_data::{Address as SOLAddress, Uint},
    SolType,
};
use stylus_proc::SolidityError;
use stylus_sdk::{
    call::{call, Call},
    contract::address,
    evm::gas_left,
    function_selector, msg,
    storage::TopLevelStorage,
    types::AddressVM,
};

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

/// Wrappers around ERC-20 operations that throw on failure (when the token
/// contract returns false). Tokens that return no value (and instead revert or
/// throw on failure) are also supported, non-reverting calls are assumed to be
/// successful.
/// To use this library you can add a `using SafeERC20 for IERC20;` statement to your contract,
/// which allows you to call the safe operations as `token.safeTransfer(...)`, etc.
pub trait SafeErc20 {
    /// The error type associated to this Safe ERC-20 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Transfer `value` amount of `token` from the calling contract to `to`. If `token` returns no value,
    /// non-reverting calls are assumed to be successful.
    fn safe_transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error>;
}

impl SafeErc20 for Erc20 {
    type Error = Error;

    fn safe_transfer(&mut self, to: Address, value: U256) -> Result<(), Error> {
        type TransferType = (SOLAddress, Uint<256>);
        let tx_data = (to, value);
        let data = TransferType::abi_encode_params(&tx_data);
        // Get function selector
        let hashed_function_selector =
            function_selector!("transfer", Address, U256);
        // Combine function selector and input data (use abi_packed way)
        let calldata = [&hashed_function_selector[..4], &data].concat();
        self.call_optional_return(calldata)
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc20 {}

impl Erc20 {
    /// Imitates a Solidity high-level call (i.e. a regular function call to a contract), relaxing the requirement
    /// on the return value: the return value is optional (but if data is returned, it must not be false).
    /// @param token The token targeted by the call.
    /// @param data The call data (encoded using abi.encode or one of its variants).
    ///
    /// This is a variant of {_callOptionalReturnBool} that reverts if call fails to meet the requirements.
    fn call_optional_return(&mut self, data: Vec<u8>) -> Result<(), Error> {
        let result = call(
            Call::new_in(self).value(msg::value()).gas(gas_left()),
            address(),
            data.as_slice(),
        );
        match result {
            Ok(data) => {
                if data.is_empty() && !Address::has_code(&address()) {
                    return Err(Error::SafeErc20FailedOperation(
                        SafeErc20FailedOperation { token: address() },
                    ));
                }
            }
            Err(_) => {
                return Err(Error::SafeErc20FailedOperation(
                    SafeErc20FailedOperation { token: address() },
                ))
            }
        }
        Ok(())
    }
}
