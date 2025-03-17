#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::ownable::{self, Ownable},
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
struct OwnableExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    ownable: Ownable,
}

#[public]
#[inherit(Erc20, Ownable)]
impl OwnableExample {
    #[constructor]
    fn constructor(&mut self, initial_owner: Address) -> Result<(), Error> {
        Ok(self.ownable.constructor(initial_owner)?)
    }

    fn transfer(&mut self, to: Address, value: U256) -> Result<(), Error> {
        self.ownable.only_owner()?;
        self.erc20.transfer(to, value)?;
        Ok(())
    }
}
