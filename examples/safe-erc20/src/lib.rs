#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;


use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::utils::safe_erc20::SafeErc20;
use stylus_sdk::prelude::*;

#[derive(Clone)]
pub struct SafeErc20Example {
    safe_erc20: SafeErc20,
}

#[inherit(SafeErc20)]
impl SafeErc20Example {}

#[external]
impl SafeErc20Example {
    pub fn transfer_and_call(

        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        self.transfer_and_call_relaxed(token, to, value, data)
    }

    pub fn transfer_from_and_call(

        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        self.transfer_from_and_call_relaxed(token, from, to, value, data)
    }

    pub fn approve_and_call(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        self.approve_and_call_relaxed(token, to, value, data)
    }

    pub fn try_transfer(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.try_safe_transfer(token, to, value)
    }

    pub fn try_transfer_from(
        &mut self,
        token: Address,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.try_safe_transfer_from(token, from, to, value)
    }
}
