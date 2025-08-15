//! Wrappers around ERC-20 operations that throw on failure (when the token
//! contract returns false).
//!
//! Tokens that return no value (and instead revert or
//! throw on failure) are also supported, non-reverting calls are assumed to be
//! successful.
//!
//! To use this library, you can add a `#[implements(ISafeErc20<Error =
//! Error>)]` attribute to your contract, which allows you to call the safe
//! operations as `contract.safe_transfer(token_addr, ...)`, etc.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol_data::Bool, SolCall, SolType};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    abi::Bytes,
    call::{MethodError, RawCall},
    contract::address,
    function_selector,
    prelude::*,
    types::AddressVM,
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
    /// An operation with an ERC-20 token failed.
    SafeErc20FailedOperation(SafeErc20FailedOperation),
    /// Indicates a failed [`ISafeErc20::safe_decrease_allowance`] request.
    SafeErc20FailedDecreaseAllowance(SafeErc20FailedDecreaseAllowance),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

use token::{IERC1363, IERC20};
mod token {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]
    alloy_sol_types::sol! {
        /// Interface of the ERC-20 token.
        interface IERC20 {
            function allowance(address owner, address spender) external view returns (uint256);
            function approve(address spender, uint256 value) external returns (bool);
            function transfer(address to, uint256 value) external returns (bool);
            function transferFrom(address from, address to, uint256 value) external returns (bool);
        }

        interface IERC1363 {
            function transferAndCall(address to, uint256 value, bytes calldata data) external returns (bool);
            function transferFromAndCall(address from, address to, uint256 value, bytes calldata data) external returns (bool);
            function approveAndCall(address spender, uint256 value, bytes calldata data) external returns (bool);
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

    /// Variant of [`Self::safe_transfer`] that returns a `bool` instead of
    /// reverting if the operation is not successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    fn try_safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> bool;

    /// Variant of [`Self::safe_transfer_from`] that returns a `bool` instead of
    /// reverting if the operation is not successful.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    fn try_safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> bool;

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
    /// * If increased allowance exceeds [`U256::MAX`].
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

    /// Performs an `IERC1363::transferAndCall`, with a fallback to the simple
    /// [`crate::token::erc20::IErc20::transfer`] if the target has no code.
    ///
    /// This can be used to implement an [`crate::token::erc721::Erc721`] like
    /// safe transfer that rely on `IERC1363` checks when targeting contracts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   `IERC1363`.
    ///
    /// # Errors
    ///
    ///  * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///    contract, the contract fails to execute the call or the call returns
    ///    value that is not `true`.
    fn transfer_and_call_relaxed(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error>;

    /// Performs an `IERC1363::transferFromAndCall`, with a fallback to the
    /// simple `IERC20::transferFrom` if the target has no code.
    ///
    /// This can be used to implement an [`crate::token::erc721::Erc721`] like
    /// safe transfer that rely on `IERC1363` checks when
    /// targeting contracts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token` - Address of the ERC-20 token contract.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   `IERC1363`.
    ///
    /// # Errors
    ///
    ///  * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///    contract , the contract fails to execute the call or the call returns
    ///    value that is not `true`.
    fn transfer_from_and_call_relaxed(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error>;

    /// Performs an `IERC1363::approveAndCall`, with a fallback to the
    /// simple [`crate::token::erc20::IErc20::approve`] if the target has no
    /// code.
    ///
    /// This can be used to implement an [`crate::token::erc721::Erc721`] like
    /// safe transfer that rely on `IERC1363` checks when
    /// targeting contracts.
    ///
    /// NOTE: When the recipient address (`spender`) has no code (i.e. is an
    /// EOA), this function behaves as [`Self::force_approve`]. Opposedly,
    /// when the recipient address (`spender`) has code, this function only
    /// attempts to call `IERC1363::approveAndCall` once without retrying,
    /// and relies on the returned value to be `true`.
    ///
    /// # Errors
    ///
    ///  * [`Error::SafeErc20FailedOperation`] - If the `token` address is not a
    ///    contract , the contract fails to execute the call or the call returns
    ///    value that is not `true`.
    fn approve_and_call_relaxed(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error>;
}

#[public]
#[implements(ISafeErc20<Error = Error>)]
impl SafeErc20 {}

#[public]
impl ISafeErc20 for SafeErc20 {
    type Error = Error;

    fn safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let call = IERC20::transferCall { to, value };

        Self::call_optional_return(token, &call)
    }

    fn safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        let call = IERC20::transferFromCall { from, to, value };

        Self::call_optional_return(token, &call)
    }

    fn try_safe_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> bool {
        self.safe_transfer(token, to, value).is_ok()
    }

    fn try_safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> bool {
        self.safe_transfer_from(token, from, to, value).is_ok()
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
        let approve_call = IERC20::approveCall { spender, value };

        // Try performing the approval with the desired value.
        if Self::call_optional_return(token, &approve_call).is_ok() {
            return Ok(());
        }

        // If that fails, reset the allowance to zero, then retry the desired
        // approval.
        let reset_approval_call =
            IERC20::approveCall { spender, value: U256::ZERO };
        Self::call_optional_return(token, &reset_approval_call)?;
        Self::call_optional_return(token, &approve_call)
    }

    fn transfer_and_call_relaxed(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        if !to.has_code() {
            return self.safe_transfer(token, to, value);
        }

        let call = IERC1363::transferAndCallCall {
            to,
            value,
            data: data.to_vec().into(),
        };

        Self::call_optional_return(token, &call)
    }

    fn transfer_from_and_call_relaxed(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        if !to.has_code() {
            return self.safe_transfer_from(token, from, to, value);
        }

        let call = IERC1363::transferFromAndCallCall {
            from,
            to,
            value,
            data: data.to_vec().into(),
        };

        Self::call_optional_return(token, &call)
    }

    fn approve_and_call_relaxed(
        &mut self,
        token: Address,
        spender: Address,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        if !spender.has_code() {
            return self.force_approve(token, spender, value);
        }

        let call = IERC1363::approveAndCallCall {
            spender,
            value,
            data: data.to_vec().into(),
        };

        Self::call_optional_return(token, &call)
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
        let result = unsafe {
            RawCall::new()
                .limit_return_data(0, BOOL_TYPE_SIZE)
                .flush_storage_cache()
                .call(token, &call.abi_encode())
        };

        match result {
            Ok(data)
                if (data.is_empty() && token.has_code())
                    || (!data.is_empty()
                        && Bool::abi_decode(&data, true)
                            .is_ok_and(|success| success)) =>
            {
                Ok(())
            }
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
    /// * [`Error::SafeErc20FailedOperation`] - If the contract fails to read
    ///   `spender`'s allowance.
    fn allowance(token: Address, spender: Address) -> Result<U256, Error> {
        let call = IERC20::allowanceCall { owner: address(), spender };
        let result = unsafe {
            RawCall::new()
                .limit_return_data(0, U256::BITS / 8)
                .flush_storage_cache()
                .call(token, &call.abi_encode())
                .map_err(|_| {
                    Error::SafeErc20FailedOperation(SafeErc20FailedOperation {
                        token,
                    })
                })?
        };

        Ok(U256::from_be_slice(&result))
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::Contract;
    use stylus_sdk::{alloy_primitives::Address, msg};

    use super::*;
    use crate::token::erc20::{Approval, Erc20, IErc20, Transfer};

    #[storage]
    #[entrypoint]
    struct SafeErc20Example {
        safe_erc20: SafeErc20,
    }

    #[public]
    #[implements(ISafeErc20<Error = Error>)]
    impl SafeErc20Example {}

    #[public]
    impl ISafeErc20 for SafeErc20Example {
        type Error = Error;

        fn safe_transfer(
            &mut self,
            token: Address,
            to: Address,
            value: U256,
        ) -> Result<(), Self::Error> {
            self.safe_erc20.safe_transfer(token, to, value)
        }

        fn safe_transfer_from(
            &mut self,
            token: Address,
            from: Address,
            to: Address,
            value: U256,
        ) -> Result<(), Self::Error> {
            self.safe_erc20.safe_transfer_from(token, from, to, value)
        }

        fn try_safe_transfer(
            &mut self,
            token: Address,
            to: Address,
            value: U256,
        ) -> bool {
            self.safe_erc20.try_safe_transfer(token, to, value)
        }

        fn try_safe_transfer_from(
            &mut self,
            token: Address,
            from: Address,
            to: Address,
            value: U256,
        ) -> bool {
            self.safe_erc20.try_safe_transfer_from(token, from, to, value)
        }

        fn safe_increase_allowance(
            &mut self,
            token: Address,
            spender: Address,
            value: U256,
        ) -> Result<(), Self::Error> {
            self.safe_erc20.safe_increase_allowance(token, spender, value)
        }

        fn safe_decrease_allowance(
            &mut self,
            token: Address,
            spender: Address,
            requested_decrease: U256,
        ) -> Result<(), Self::Error> {
            self.safe_erc20.safe_decrease_allowance(
                token,
                spender,
                requested_decrease,
            )
        }

        fn force_approve(
            &mut self,
            token: Address,
            spender: Address,
            value: U256,
        ) -> Result<(), Self::Error> {
            self.safe_erc20.force_approve(token, spender, value)
        }

        fn transfer_and_call_relaxed(
            &mut self,
            token: Address,
            to: Address,
            value: U256,
            data: Bytes,
        ) -> Result<(), Self::Error> {
            self.safe_erc20.transfer_and_call_relaxed(token, to, value, data)
        }

        fn transfer_from_and_call_relaxed(
            &mut self,
            token: Address,
            from: Address,
            to: Address,
            value: U256,
            data: Bytes,
        ) -> Result<(), Self::Error> {
            self.safe_erc20
                .transfer_from_and_call_relaxed(token, from, to, value, data)
        }

        fn approve_and_call_relaxed(
            &mut self,
            token: Address,
            spender: Address,
            value: U256,
            data: Bytes,
        ) -> Result<(), Self::Error> {
            self.safe_erc20
                .approve_and_call_relaxed(token, spender, value, data)
        }
    }

    #[motsu::test]
    fn try_safe_transfer_returns_true(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(1);
        erc20.sender(alice)._mint(contract.address(), value).unwrap();

        let balance = erc20.sender(alice).balance_of(contract.address());
        assert_eq!(balance, value);
        let balance = erc20.sender(alice).balance_of(alice);
        assert_eq!(balance, U256::ZERO);

        let result =
            contract.sender(alice).try_safe_transfer(token, alice, value);
        assert!(result);

        let balance = erc20.sender(alice).balance_of(contract.address());
        assert_eq!(balance, U256::ZERO);
        let balance = erc20.sender(alice).balance_of(alice);
        assert_eq!(balance, value);
    }

    #[motsu::test]
    fn try_safe_transfer_from_returns_true(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(1);
        erc20.sender(alice)._mint(alice, value).unwrap();
        erc20.sender(alice).approve(contract.address(), value).unwrap();

        let balance = erc20.sender(alice).balance_of(alice);
        assert_eq!(balance, value);
        let balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(balance, U256::ZERO);

        let result = contract
            .sender(alice)
            .try_safe_transfer_from(token, alice, bob, value);
        assert!(result);

        let balance = erc20.sender(alice).balance_of(alice);
        assert_eq!(balance, U256::ZERO);
        let balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(balance, value);
    }

    #[motsu::test]
    fn safe_transfer_does_not_revert(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(1);

        // Mint tokens to the SafeErc20Example contract so it can transfer out.
        erc20.sender(alice)._mint(contract.address(), U256::from(10)).unwrap();

        let initial_safe_erc20_balance =
            erc20.sender(alice).balance_of(contract.address());
        let initial_bob_balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(initial_safe_erc20_balance, U256::from(10));
        assert_eq!(initial_bob_balance, U256::ZERO);

        contract.sender(alice).safe_transfer(token, bob, value).unwrap();

        erc20.assert_emitted(&Transfer {
            from: contract.address(),
            to: bob,
            value,
        });

        let safe_erc20_balance =
            erc20.sender(alice).balance_of(contract.address());
        let bob_balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(safe_erc20_balance, initial_safe_erc20_balance - value);
        assert_eq!(bob_balance, initial_bob_balance + value);
    }

    #[motsu::test]
    fn safe_transfer_reverts(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(1);

        let initial_safe_erc20_balance =
            erc20.sender(alice).balance_of(contract.address());
        let initial_bob_balance = erc20.sender(alice).balance_of(bob);

        let err = contract
            .sender(alice)
            .safe_transfer(token, bob, value)
            .unwrap_err();
        assert!(matches!(err, Error::SafeErc20FailedOperation(_)));

        let safe_erc20_balance =
            erc20.sender(alice).balance_of(contract.address());
        let bob_balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(initial_safe_erc20_balance, safe_erc20_balance);
        assert_eq!(initial_bob_balance, bob_balance);
    }

    #[motsu::test]
    fn safe_transfer_from_does_not_revert(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(1);

        erc20.sender(alice)._mint(alice, U256::from(10)).unwrap();
        erc20.sender(alice).approve(contract.address(), value).unwrap();

        let initial_alice_balance = erc20.sender(alice).balance_of(alice);
        let initial_bob_balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(initial_alice_balance, U256::from(10));
        assert_eq!(initial_bob_balance, U256::ZERO);

        contract
            .sender(alice)
            .safe_transfer_from(token, alice, bob, value)
            .unwrap();

        erc20.assert_emitted(&Transfer { from: alice, to: bob, value });

        let alice_balance = erc20.sender(alice).balance_of(alice);
        let bob_balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(alice_balance, initial_alice_balance - value);
        assert_eq!(bob_balance, initial_bob_balance + value);
    }

    #[motsu::test]
    fn safe_transfer_from_reverts(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(1);

        erc20.sender(alice).approve(contract.address(), value).unwrap();

        let initial_alice_balance = erc20.sender(alice).balance_of(alice);
        let initial_bob_balance = erc20.sender(alice).balance_of(bob);

        let err = contract
            .sender(alice)
            .safe_transfer_from(token, alice, bob, value)
            .unwrap_err();
        assert!(matches!(err, Error::SafeErc20FailedOperation(_)));

        let alice_balance = erc20.sender(alice).balance_of(alice);
        let bob_balance = erc20.sender(alice).balance_of(bob);
        assert_eq!(initial_alice_balance, alice_balance);
        assert_eq!(initial_bob_balance, bob_balance);
    }

    // Approval and allowance tests.
    #[motsu::test]
    fn force_approve_sets_allowance_from_zero(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice;
        // Precondition: 0 allowance.
        let initial =
            erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(initial, U256::ZERO);

        let value = U256::from(100);
        contract.sender(alice).force_approve(token, spender, value).unwrap();

        erc20.assert_emitted(&Approval {
            owner: contract.address(),
            spender,
            value,
        });
        let after = erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(after, value);
    }

    #[motsu::test]
    fn force_approve_updates_non_zero_allowance(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice;
        // Set initial non-zero allowance.
        contract
            .sender(alice)
            .force_approve(token, spender, U256::from(7))
            .unwrap();
        erc20.assert_emitted(&Approval {
            owner: contract.address(),
            spender,
            value: U256::from(7),
        });

        // Update to a different value.
        let new_value = U256::from(3);
        contract
            .sender(alice)
            .force_approve(token, spender, new_value)
            .unwrap();
        erc20.assert_emitted(&Approval {
            owner: contract.address(),
            spender,
            value: new_value,
        });
        let after = erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(after, new_value);
    }

    #[motsu::test]
    fn safe_increase_allowance_increases(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice;
        // Start from zero.
        let inc = U256::from(10);
        contract
            .sender(alice)
            .safe_increase_allowance(token, spender, inc)
            .unwrap();
        // The event has the new allowance value.
        erc20.assert_emitted(&Approval {
            owner: contract.address(),
            spender,
            value: inc,
        });
        let after = erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(after, inc);
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for allowance"]
    fn safe_increase_allowance_overflow_panics(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice;
        // Set to max then try to increase.
        contract
            .sender(alice)
            .force_approve(token, spender, U256::MAX)
            .unwrap();
        contract
            .sender(alice)
            .safe_increase_allowance(token, spender, U256::from(1))
            .unwrap();
    }

    #[motsu::test]
    fn safe_decrease_allowance_errors_if_below_zero(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice;
        // Current allowance: 0.
        let err = contract
            .sender(alice)
            .safe_decrease_allowance(token, spender, U256::from(1))
            .unwrap_err();
        assert!(matches!(err, Error::SafeErc20FailedDecreaseAllowance(_)));
        // Stays zero.
        let after = erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(after, U256::ZERO);
    }

    #[motsu::test]
    fn safe_decrease_allowance_decreases(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice;
        // Set to 10 then decrease by 3.
        contract
            .sender(alice)
            .force_approve(token, spender, U256::from(10))
            .unwrap();
        contract
            .sender(alice)
            .safe_decrease_allowance(token, spender, U256::from(3))
            .unwrap();
        erc20.assert_emitted(&Approval {
            owner: contract.address(),
            spender,
            value: U256::from(7),
        });
        let after = erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(after, U256::from(7));
    }

    // --- ERC1363 relaxed-call tests ---

    /// Dummy target contracts to ensure `has_code()` is true for
    /// receiver/spender.
    #[storage]
    struct DummyReceiver;
    unsafe impl TopLevelStorage for DummyReceiver {}

    #[public]
    impl DummyReceiver {}

    #[storage]
    struct DummySpender;
    unsafe impl TopLevelStorage for DummySpender {}

    #[public]
    impl DummySpender {}

    /// ERC1363 token that returns true for all 1363 methods.
    #[storage]
    struct Erc1363TokenOk;
    unsafe impl TopLevelStorage for Erc1363TokenOk {}

    #[public]
    #[allow(non_snake_case)]
    impl Erc1363TokenOk {
        fn transferAndCall(
            &mut self,
            _to: Address,
            _value: U256,
            _data: Bytes,
        ) -> Result<bool, Vec<u8>> {
            Ok(true)
        }

        fn transferFromAndCall(
            &mut self,
            _from: Address,
            _to: Address,
            _value: U256,
            _data: Bytes,
        ) -> Result<bool, Vec<u8>> {
            Ok(true)
        }

        fn approveAndCall(
            &mut self,
            _spender: Address,
            _value: U256,
            _data: Bytes,
        ) -> Result<bool, Vec<u8>> {
            Ok(true)
        }
    }

    /// ERC1363 token that returns false for all 1363 methods.
    #[storage]
    struct Erc1363TokenFalse;
    unsafe impl TopLevelStorage for Erc1363TokenFalse {}

    #[public]
    #[allow(non_snake_case)]
    impl Erc1363TokenFalse {
        fn transferAndCall(
            &mut self,
            _to: Address,
            _value: U256,
            _data: Bytes,
        ) -> Result<bool, Vec<u8>> {
            Ok(false)
        }

        fn transferFromAndCall(
            &mut self,
            _from: Address,
            _to: Address,
            _value: U256,
            _data: Bytes,
        ) -> Result<bool, Vec<u8>> {
            Ok(false)
        }

        fn approveAndCall(
            &mut self,
            _spender: Address,
            _value: U256,
            _data: Bytes,
        ) -> Result<bool, Vec<u8>> {
            Ok(false)
        }
    }

    // transfer_and_call_relaxed
    #[motsu::test]
    fn transfer_and_call_relaxed_falls_back_to_transfer_on_eoa(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(5);
        let data: Bytes = vec![].into();

        // Fund SafeErc20Example.
        erc20.sender(alice)._mint(contract.address(), U256::from(10)).unwrap();

        contract
            .sender(alice)
            .transfer_and_call_relaxed(token, bob, value, data)
            .unwrap();

        erc20.assert_emitted(&Transfer {
            from: contract.address(),
            to: bob,
            value,
        });
    }

    #[motsu::test]
    fn transfer_and_call_relaxed_calls_erc1363_when_to_has_code(
        contract: Contract<SafeErc20Example>,
        token1363: Contract<Erc1363TokenOk>,
        receiver: Contract<DummyReceiver>,
        alice: Address,
    ) {
        let token = token1363.address();
        let to = receiver.address();
        let value = U256::from(1);
        let data: Bytes = vec![].into();

        // Since `to` has code, path calls IERC1363::transferAndCall; token
        // returns `true`.
        contract
            .sender(alice)
            .transfer_and_call_relaxed(token, to, value, data)
            .unwrap();
    }

    #[motsu::test]
    fn transfer_and_call_relaxed_reverts_when_token_returns_false(
        contract: Contract<SafeErc20Example>,
        token1363: Contract<Erc1363TokenFalse>,
        receiver: Contract<DummyReceiver>,
        alice: Address,
    ) {
        let token = token1363.address();
        let to = receiver.address();
        let value = U256::from(1);
        let data: Bytes = vec![].into();

        let err = contract
            .sender(alice)
            .transfer_and_call_relaxed(token, to, value, data)
            .unwrap_err();
        assert!(matches!(err, Error::SafeErc20FailedOperation(_)));
    }

    // transfer_from_and_call_relaxed
    #[motsu::test]
    fn transfer_from_and_call_relaxed_falls_back_to_transfer_from_on_eoa(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let token = erc20.address();
        let value = U256::from(2);
        let data: Bytes = vec![].into();

        // Fund Alice and approve the SafeErc20Example.
        erc20.sender(alice)._mint(alice, U256::from(10)).unwrap();
        erc20.sender(alice).approve(contract.address(), value).unwrap();

        contract
            .sender(alice)
            .transfer_from_and_call_relaxed(token, alice, bob, value, data)
            .unwrap();

        erc20.assert_emitted(&Transfer { from: alice, to: bob, value });
    }

    #[motsu::test]
    fn transfer_from_and_call_relaxed_calls_erc1363_when_to_has_code(
        contract: Contract<SafeErc20Example>,
        token1363: Contract<Erc1363TokenOk>,
        receiver: Contract<DummyReceiver>,
        alice: Address,
    ) {
        let token = token1363.address();
        let to = receiver.address();
        let value = U256::from(3);
        let data: Bytes = vec![].into();

        contract
            .sender(alice)
            .transfer_from_and_call_relaxed(token, alice, to, value, data)
            .unwrap();
    }

    #[motsu::test]
    fn transfer_from_and_call_relaxed_reverts_when_token_returns_false(
        contract: Contract<SafeErc20Example>,
        token1363: Contract<Erc1363TokenFalse>,
        receiver: Contract<DummyReceiver>,
        alice: Address,
    ) {
        let token = token1363.address();
        let to = receiver.address();
        let value = U256::from(1);
        let data: Bytes = vec![].into();

        let err = contract
            .sender(alice)
            .transfer_from_and_call_relaxed(token, alice, to, value, data)
            .unwrap_err();
        assert!(matches!(err, Error::SafeErc20FailedOperation(_)));
    }

    // approve_and_call_relaxed
    #[motsu::test]
    fn approve_and_call_relaxed_falls_back_to_force_approve_on_eoa(
        contract: Contract<SafeErc20Example>,
        erc20: Contract<Erc20>,
        alice: Address,
    ) {
        let token = erc20.address();
        let spender = alice; // EOA
        let value = U256::from(11);
        let data: Bytes = vec![].into();

        contract
            .sender(alice)
            .approve_and_call_relaxed(token, spender, value, data)
            .unwrap();

        erc20.assert_emitted(&Approval {
            owner: contract.address(),
            spender,
            value,
        });
        let after = erc20.sender(alice).allowance(contract.address(), spender);
        assert_eq!(after, value);
    }

    #[motsu::test]
    fn approve_and_call_relaxed_calls_erc1363_when_spender_has_code(
        contract: Contract<SafeErc20Example>,
        token1363: Contract<Erc1363TokenOk>,
        spender: Contract<DummySpender>,
        alice: Address,
    ) {
        let token = token1363.address();
        let sp = spender.address();
        let value = U256::from(7);
        let data: Bytes = vec![].into();

        contract
            .sender(alice)
            .approve_and_call_relaxed(token, sp, value, data)
            .unwrap();
    }

    #[motsu::test]
    fn approve_and_call_relaxed_reverts_when_token_returns_false(
        contract: Contract<SafeErc20Example>,
        token1363: Contract<Erc1363TokenFalse>,
        spender: Contract<DummySpender>,
        alice: Address,
    ) {
        let token = token1363.address();
        let sp = spender.address();
        let value = U256::from(1);
        let data: Bytes = vec![].into();

        let err = contract
            .sender(alice)
            .approve_and_call_relaxed(token, sp, value, data)
            .unwrap_err();
        assert!(matches!(err, Error::SafeErc20FailedOperation(_)));
    }

    // Mock ERC20-like contract that reverts on `allowance` calls.
    #[storage]
    struct RevertingAllowanceToken;

    unsafe impl TopLevelStorage for RevertingAllowanceToken {}

    #[public]
    impl RevertingAllowanceToken {
        // External signature matches `IERC20.allowance(owner, spender) ->
        // uint256`. Reverting causes a revert so the `RawCall` in
        // `SafeErc20::allowance` fails.
        fn allowance(
            &self,
            _owner: Address,
            _spender: Address,
        ) -> Result<U256, Vec<u8>> {
            Err("revert".into())
        }
    }

    #[motsu::test]
    fn safe_increase_allowance_reverts_on_allowance_call_error(
        contract: Contract<SafeErc20Example>,
        bad_token: Contract<RevertingAllowanceToken>,
        alice: Address,
    ) {
        let token = bad_token.address();
        let err = contract
            .sender(alice)
            .safe_increase_allowance(token, alice, U256::from(1))
            .unwrap_err();
        assert!(
            matches!(err, Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }) if token == bad_token.address())
        );
    }

    #[motsu::test]
    fn safe_decrease_allowance_reverts_on_allowance_call_error(
        contract: Contract<SafeErc20Example>,
        bad_token: Contract<RevertingAllowanceToken>,
        alice: Address,
    ) {
        let token = bad_token.address();
        let err = contract
            .sender(alice)
            .safe_decrease_allowance(token, alice, U256::from(1))
            .unwrap_err();
        assert!(
            matches!(err, Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }) if token == bad_token.address())
        );
    }

    // Mock ERC20-like contract that panics on `allowance` calls.
    #[storage]
    struct PanickingAllowanceToken;

    unsafe impl TopLevelStorage for PanickingAllowanceToken {}

    #[public]
    impl PanickingAllowanceToken {
        // External signature matches IERC20.allowance(owner, spender) ->
        // uint256 Panicking causes a revert so the RawCall in
        // SafeErc20::allowance fails.
        fn allowance(&self, _owner: Address, _spender: Address) -> U256 {
            panic!("revert");
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "See: https://github.com/OpenZeppelin/stylus-test-helpers/issues/116"]
    fn safe_increase_allowance_reverts_on_allowance_call_panic(
        contract: Contract<SafeErc20Example>,
        bad_token: Contract<PanickingAllowanceToken>,
        alice: Address,
    ) {
        let token = bad_token.address();
        let err = contract
            .sender(alice)
            .safe_increase_allowance(token, alice, U256::from(1))
            .unwrap_err();
        assert!(
            matches!(err, Error::SafeErc20FailedOperation(SafeErc20FailedOperation { token }) if token == bad_token.address())
        );
    }

    /// Mock contract with USDT-like approval behavior.
    #[storage]
    struct USDTLikeToken {
        erc20: Erc20,
    }

    unsafe impl TopLevelStorage for USDTLikeToken {}

    #[public]
    impl USDTLikeToken {
        fn allowance(&self, owner: Address, spender: Address) -> U256 {
            self.erc20.allowance(owner, spender)
        }

        fn approve(
            &mut self,
            spender: Address,
            amount: U256,
        ) -> Result<bool, Vec<u8>> {
            let owner = msg::sender();
            if amount.is_zero()
                || self.erc20.allowance(owner, spender).is_zero()
            {
                return Ok(self.erc20.approve(spender, amount)?);
            }

            Err("USDT approval failure".into())
        }
    }

    #[motsu::test]
    fn safe_increase_allowance_usdt_like(
        contract: Contract<SafeErc20Example>,
        usdt_like_token: Contract<USDTLikeToken>,
        alice: Address,
    ) {
        let token = usdt_like_token.address();
        let spender = alice;

        // Set to 10.
        contract
            .sender(alice)
            .force_approve(token, spender, U256::from(10))
            .unwrap();
        let before = usdt_like_token
            .sender(alice)
            .allowance(contract.address(), spender);
        assert_eq!(before, U256::from(10));

        // Then increase to 20.
        contract
            .sender(alice)
            .safe_increase_allowance(token, spender, U256::from(10))
            .unwrap();
        let after = usdt_like_token
            .sender(alice)
            .allowance(contract.address(), spender);
        assert_eq!(after, U256::from(20));
    }
}
