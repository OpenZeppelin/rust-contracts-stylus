use alloc::vec::Vec;

use alloy_primitives::Address;
pub use sol::*;
use stylus_sdk::{
    call::{self, Call, MethodError},
    prelude::*,
};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// There's no code at `target` (it is not a contract).
        ///
        /// * `target` - Address of the target contract.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error AddressEmptyCode(address target);

        /// A call to an address target failed. The target may have reverted.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error FailedCall();
    }
}

/// An [`AddressUtils`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// There's no code at `target` (it is not a contract).
    EmptyCode(AddressEmptyCode),
    /// A call to an address target failed. The target may have reverted.
    FailedCall(FailedCall),
    /// A call to an address target failed. The target may have reverted
    /// with a reason.
    StylusError(stylus_sdk::call::Error),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// A collection of utilities for working with [`Address`].
pub struct AddressUtils;

impl AddressUtils {
    /// Performs a delegate call to `target` with the given `data`.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable access to the contract's state.
    /// * `target` - The address of the target contract.
    /// * `data` - The data to pass to the target contract.
    ///
    /// # Errors
    ///
    /// * [`Error::FailedCall`] - If the call to the target contract fails
    ///   without a revert reason.
    /// * [`Error::StylusError`] - If the call to the target contract fails with
    ///   a revert reason or if the call fails for any other reason.
    /// * [`Error::EmptyCode`] - If the target contract has no code.
    pub fn function_delegate_call(
        context: &mut impl TopLevelStorage,
        target: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let result =
            unsafe { call::delegate_call(Call::new_in(context), target, data) };
        Self::verify_call_result_from_target(target, result)
    }

    /// Helper function to verify that a low level call to smart-contract was
    /// successful.
    ///
    /// Reverts if the target was not a contract or if the call fails for any
    /// other reason. Bubbles up the revert reason (falling back to
    /// [`Error::FailedCall`]) in case of an unsuccessful call.
    ///
    /// # Arguments
    ///
    /// * `target` - The address of the target contract.
    /// * `result` - The result of the call.
    ///
    /// # Errors
    ///
    /// * [`Error::EmptyCode`] - If the target contract has no code.
    /// * [`Error::StylusError`] - If the call to the target contract fails with
    ///   a revert reason or if the call fails for any other reason.
    /// * [`Error::FailedCall`] - If the call to the target contract fails
    ///   without a revert reason.
    pub fn verify_call_result_from_target(
        target: Address,
        result: Result<Vec<u8>, stylus_sdk::call::Error>,
    ) -> Result<Vec<u8>, Error> {
        match result {
            Ok(returndata) => {
                if returndata.is_empty() && !target.has_code() {
                    return Err(AddressEmptyCode { target }.into());
                }
                Ok(returndata)
            }
            Err(e) => Self::revert(e),
        }
    }
}

impl AddressUtils {
    /// Reverts with `error` if revert reason exists. Otherwise reverts with
    /// [`Error::FailedCall`].
    ///
    /// This behavior is aligned with Solidity implementation of
    /// [Address.sol].
    ///
    /// [Address.sol]: https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/Address.sol
    fn revert(error: stylus_sdk::call::Error) -> Result<Vec<u8>, Error> {
        match &error {
            stylus_sdk::call::Error::Revert(data) if data.is_empty() => {
                Err(FailedCall {}.into())
            }
            _ => Err(Error::StylusError(error)),
        }
    }
}
