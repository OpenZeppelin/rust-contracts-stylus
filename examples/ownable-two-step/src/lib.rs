#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::{ownable, ownable_two_step::Ownable2Step},
    token::erc20::{self, Erc20, IErc20},
};
use stylus_sdk::prelude::*;

#[derive(SolidityError, Debug)]
enum Error {
    Erc20(erc20::Error),
    Ownable(ownable::Error),
}

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
    pub fn transfer(&mut self, to: Address, value: U256) -> Result<(), Error> {
        self.ownable.only_owner()?;
        self.erc20.transfer(to, value)?;
        Ok(())
    }
}
