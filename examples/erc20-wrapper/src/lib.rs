#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256, U8};
use openzeppelin_stylus::token::erc20::{
    extensions::{wrapper, Erc20Wrapper, IErc20Wrapper},
    Erc20, IErc20,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc20WrapperExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    erc20_wrapper: Erc20Wrapper,
}

#[public]
#[implements(IErc20<Error = wrapper::Error>, IErc20Wrapper<Error = wrapper::Error>)]
impl Erc20WrapperExample {
    #[constructor]
    fn constructor(
        &mut self,
        underlying_token: Address,
        decimals: U8,
    ) -> Result<(), wrapper::Error> {
        self.erc20_wrapper.constructor(underlying_token)?;
        self.erc20_wrapper.underlying_decimals.set(decimals);
        Ok(())
    }
}

#[public]
impl IErc20 for Erc20WrapperExample {
    type Error = wrapper::Error;

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20>::Error> {
        Ok(self.erc20.transfer(to, value)?)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20>::Error> {
        Ok(self.erc20.approve(spender, value)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20>::Error> {
        Ok(self.erc20.transfer_from(from, to, value)?)
    }
}

#[public]
impl IErc20Wrapper for Erc20WrapperExample {
    type Error = wrapper::Error;

    fn underlying(&self) -> Address {
        self.erc20_wrapper.underlying()
    }

    fn decimals(&self) -> U8 {
        self.erc20_wrapper.decimals()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20Wrapper>::Error> {
        self.erc20_wrapper.deposit_for(account, value, &mut self.erc20)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<bool, <Self as IErc20Wrapper>::Error> {
        self.erc20_wrapper.withdraw_to(account, value, &mut self.erc20)
    }
}
