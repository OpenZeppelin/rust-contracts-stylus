#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::ownable_two_step::Ownable2Step,
    token::erc20::{Erc20, IErc20},
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Ownable2StepExample {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub ownable: Ownable2Step,
}

#[public]
#[inherit(Erc20, Ownable2Step)]
impl Ownable2StepExample {
    #[constructor]
    pub fn constructor(
        &mut self,
        _initial_owner: Address,
    ) -> Result<(), Vec<u8>> {
        todo!("uncomment once multiple public attributes are supported")
        // Ok(self.ownable.constructor(initial_owner)?)
    }

    pub fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.ownable._ownable.only_owner()?;
        self.erc20.transfer(to, value)?;
        Ok(())
    }
}
