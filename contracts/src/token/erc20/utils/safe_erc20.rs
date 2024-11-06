//! Wrappers around ERC-20 operations that throw on failure (when the token
//! contract returns false). Tokens that return no value (and instead revert or
//! throw on failure) are also supported, non-reverting calls are assumed to be
//! successful.
//! To use this library you can add a `#[inherit(SafeErc20)]` attribute to
//! your contract, which allows you to call the safe operations as
//! `contract.safe_transfer(token_addr, ...)`, etc.
use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolValue};
use stylus_sdk::{
    call::{Call, RawCall},
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

pub use token::IErc20;
#[allow(missing_docs)]
mod token {
    stylus_sdk::stylus_proc::sol_interface! {
        /// Interface of the ERC-20 standard as defined in the ERC.
        interface IErc20 {
            function allowance(address owner, address spender) external view returns (uint256);
            function approve(address spender, uint256 amount) external returns (bool);
            function transfer(address recipient, uint256 amount) external returns (bool);
            function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
        }
    }
}
sol_storage! {
    /// State of the SafeErc20 Contract.
    pub struct SafeErc20 {}
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for SafeErc20 {}

/// Required interface of an [`SafeErc20`] utility contract.
pub trait ISafeErc20 {
    /// The error type associated to this ERC-20 trait implementation.
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
        let encoded_args = (to, value).abi_encode_params();
        let selector = function_selector!("transfer", Address, U256);
        let data = [&selector[..4], &encoded_args].concat();

        self._call_optional_return(token, &data)
    }

    fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let encoded_args = (from, to, value).abi_encode_params();
        let selector =
            function_selector!("transferFrom", Address, Address, U256);
        let data = [&selector[..4], &encoded_args].concat();

        self._call_optional_return(token, &data)
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
        if self._call_optional_return(token, &approve_data).is_ok() {
            return Ok(());
        }

        // If that fails, reset allowance to zero, then retry the desired
        // approval
        let reset_data = build_approve_calldata(spender, U256::ZERO, &selector);
        self._call_optional_return(token, &reset_data)?;
        self._call_optional_return(token, &approve_data)?;

        Ok(())
    }
}

impl SafeErc20 {
    /// Imitates a Stylus high-level call, relaxing the requirement on the
    /// return value: if data is returned, it must not be `false`, otherwise
    /// calls are assumed to be successful.
    fn _call_optional_return(
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
                if (data.is_empty() && Address::has_code(&token))
                    || (!data.is_empty() && encodes_true(&data)) =>
            {
                Ok(())
            }
            _ => Err(SafeErc20FailedOperation { token }.into()),
        }
    }

    fn allowance(
        &mut self,
        token: Address,
        spender: Address,
    ) -> Result<U256, Error> {
        let erc20 = IErc20::new(token);
        let call = Call::new_in(self);
        erc20
            .allowance(call, address(), spender)
            .map_err(|_| SafeErc20FailedOperation { token }.into())
    }
}

fn encodes_true(data: &[u8]) -> bool {
    data[..data.len() - 1].iter().all(|&byte| byte == 0)
        && data[data.len() - 1] == 1
}
