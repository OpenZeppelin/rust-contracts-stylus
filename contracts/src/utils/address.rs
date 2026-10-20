//! A collection of utilities for working with [`Address`].

use alloc::vec::Vec;

use alloy_primitives::Address;
pub use sol::*;
use stylus_sdk::{call, prelude::*, stylus_core::calls::errors::MethodError};

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_types::sol;

    sol! {
        /// There's no code at `target` (it is not a contract).
        ///
        /// * `target` - Address of the target contract.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error AddressEmptyCode(address target);

        /// A call to an address target failed. The target may have reverted
        /// without a revert reason.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error FailedCall();

        /// A call to an address target failed. The target may have reverted
        /// with a reason.
        ///
        /// * `reason` - The revert reason that was returned by the call.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error FailedCallWithReason(bytes reason);
    }
}

/// An [`AddressUtils`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// There's no code at `target` (it is not a contract).
    EmptyCode(AddressEmptyCode),
    /// A call to an address target failed. The target may have reverted
    /// without a revert reason.
    FailedCall(FailedCall),
    /// A call to an address target failed. The target may have reverted
    /// with a reason.
    FailedCallWithReason(FailedCallWithReason),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// Host-aware helpers on [`Address`].
pub trait AddressExt {
    /// Returns `true` if there is code deployed at this address.
    ///
    /// # Arguments
    ///
    /// * `host` - The host VM, usually `self.vm()` from a `#[storage]` struct.
    fn has_code<H: Host + ?Sized>(self, host: &H) -> bool;
}

impl AddressExt for Address {
    fn has_code<H: Host + ?Sized>(self, host: &H) -> bool {
        host.code_size(self) > 0
    }
}

/// A collection of utilities for working with [`Address`].
pub struct AddressUtils;

impl AddressUtils {
    /// Performs a Solidity function call using a low level `delegatecall`.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable access to the contract's state.
    /// * `target` - The address of the contract to call.
    /// * `data` - The calldata to send to the target.
    ///
    /// # Errors
    ///
    /// * [`Error::EmptyCode`] - If the call returned no data and `target` has
    ///   no code.
    /// * [`Error::FailedCall`] - If the call reverted without data.
    /// * [`Error::FailedCallWithReason`] - If the call reverted with data.
    pub fn function_delegate_call<C: TopLevelStorage + HostAccess>(
        context: &mut C,
        target: Address,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let cfg = Call::new_mutating(context);
        let result =
            unsafe { call::delegate_call(context.vm(), cfg, target, data) };
        Self::verify_call_result_from_target(context.vm(), target, result)
    }

    /// Tool to verify that a low level call to smart-contract was successful,
    /// and reverts if the target was not a contract or bubbles up the revert
    /// reason (falling back to [`Error::FailedCall`]) in case of an
    /// unsuccessful call.
    ///
    /// # Arguments
    ///
    /// * `host` - The host VM, usually `self.vm()`.
    /// * `target` - The address that was called.
    /// * `result` - The result of the call.
    ///
    /// # Errors
    ///
    /// * [`Error::EmptyCode`] - If the call returned no data and `target` has
    ///   no code.
    /// * [`Error::FailedCall`] - If the call reverted without data.
    /// * [`Error::FailedCallWithReason`] - If the call reverted with data.
    pub fn verify_call_result_from_target<H, T>(
        host: &H,
        target: Address,
        result: Result<T, stylus_sdk::stylus_core::calls::errors::Error>,
    ) -> Result<T, Error>
    where
        H: Host + ?Sized,
        T: AsRef<[u8]>,
    {
        match result {
            Ok(returndata) => {
                if returndata.as_ref().is_empty() && !target.has_code(host) {
                    return Err(AddressEmptyCode { target }.into());
                }
                Ok(returndata)
            }
            Err(e) => Err(Self::revert(e)),
        }
    }
}

impl AddressUtils {
    /// Converts a call error into [`Error::FailedCall`] (empty revert data)
    /// or [`Error::FailedCallWithReason`].
    fn revert(error: stylus_sdk::stylus_core::calls::errors::Error) -> Error {
        match &error {
            stylus_sdk::stylus_core::calls::errors::Error::Revert(data)
                if data.is_empty() =>
            {
                FailedCall {}.into()
            }
            _ => FailedCallWithReason { reason: error.encode().into() }.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use testing::prelude::*;

    use super::*;

    #[test]
    fn revert_returns_failed_call() {
        let error =
            stylus_sdk::stylus_core::calls::errors::Error::Revert(vec![]);
        let result = AddressUtils::revert(error);
        assert!(matches!(result, Error::FailedCall(FailedCall {})));
    }

    #[test]
    fn revert_returns_failed_call_with_reason() {
        let error =
            stylus_sdk::stylus_core::calls::errors::Error::Revert(vec![
                1, 2, 3,
            ]);
        let result = AddressUtils::revert(error);
        assert!(matches!(
            result,
            Error::FailedCallWithReason(FailedCallWithReason { reason: _ })
        ));
    }

    #[storage]
    struct TargetMock;

    unsafe impl TopLevelStorage for TargetMock {}

    #[public]
    impl TargetMock {}

    #[testing::test]
    fn verify_call_result_from_target_returns_empty_data_when_target_has_code(
        target: Contract<TargetMock>,
    ) {
        let empty_data: Vec<u8> = vec![];
        let result = AddressUtils::verify_call_result_from_target(
            &target.env().host(),
            target.address(),
            Ok(empty_data.clone()),
        )
        .expect("should be able to verify call result");

        assert_eq!(result, empty_data);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[testing::test]
    fn verify_call_result_from_target_returns_data_when_target_has_no_code(
        env: Env,
    ) {
        let data: Vec<u8> = vec![1, 2, 3];

        let result = AddressUtils::verify_call_result_from_target(
            &env.host(),
            Address::ZERO,
            Ok(data.clone()),
        )
        .expect("should be able to verify call result");

        assert_eq!(result, data);
    }

    #[testing::test]
    fn verify_call_result_from_target_returns_address_empty_code(env: Env) {
        let result = AddressUtils::verify_call_result_from_target(
            &env.host(),
            Address::ZERO,
            Ok(vec![]),
        );
        assert!(matches!(
            result,
            Err(Error::EmptyCode(AddressEmptyCode { target: Address::ZERO }))
        ));
    }
}
