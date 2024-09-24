//! Wrappers around ERC-20 operations that throw on failure.

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use stylus_proc::{public, sol_interface, sol_storage, SolidityError};
use stylus_sdk::{call::Call, storage::TopLevelStorage, types::AddressVM};

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
        /// Moves a `value` amount of tokens from the caller's account to `to`.
        ///
        /// Returns a boolean value indicating whether the operation succeeded.
        ///
        /// Emits a {Transfer} event.
        function transfer(address to, uint256 amount) external returns (bool);

        /// Moves a `value` amount of tokens from `from` to `to` using the
        /// allowance mechanism. `value` is then deducted from the caller's
        /// allowance.
        ///
        /// Returns a boolean value indicating whether the operation succeeded.
        ///
        /// Emits a {Transfer} event.
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
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
        let erc20 = IERC20::new(token);
        let call = Call::new_in(self);

        match erc20.transfer(call, to, value) {
            Ok(data) => {
                if data && !Address::has_code(&token) {
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

    /// Transfer `value` amount of `token` from `from` to `to`, spending the approval given by `from` to the
    /// calling contract. If `token` returns no value, non-reverting calls are assumed to be successful.
    pub fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        let erc20 = IERC20::new(token);
        let call = Call::new_in(self);

        match erc20.transfer_from(call, from, to, value) {
            Ok(data) => {
                if data && !Address::has_code(&token) {
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
        todo!()
    }
}
