#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_sol_types::sol;
use openzeppelin_stylus::token::erc20::utils::safe_erc20::{
    self, ISafeErc20, SafeErc20,
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, U256},
    evm,
    prelude::*,
};

sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
    event True();
    #[derive(Debug)]
    #[allow(missing_docs)]
    event False();
}

#[entrypoint]
#[storage]
struct SafeErc20Example {
    safe_erc20: SafeErc20,
}

#[public]
#[implements(ISafeErc20<Error = safe_erc20::Error>)]
impl SafeErc20Example {}

#[public]
impl ISafeErc20 for SafeErc20Example {
    type Error = safe_erc20::Error;

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
        let result = self.safe_erc20.try_safe_transfer(token, to, value);

        // log the result so we can assert it in tests.
        if result {
            #[allow(deprecated)]
            evm::log(True {});
        } else {
            #[allow(deprecated)]
            evm::log(False {});
        }
        result
    }

    fn try_safe_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> bool {
        let result =
            self.safe_erc20.try_safe_transfer_from(token, from, to, value);

        // log the result so we can assert it in tests.
        if result {
            #[allow(deprecated)]
            evm::log(True {});
        } else {
            #[allow(deprecated)]
            evm::log(False {});
        }
        result
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
        self.safe_erc20.approve_and_call_relaxed(token, spender, value, data)
    }
}
