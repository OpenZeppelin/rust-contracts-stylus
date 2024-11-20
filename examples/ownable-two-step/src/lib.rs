#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::ownable_two_step::Ownable2Step,
    token::erc20::{Erc20, IErc20},
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Ownable2StepExample {
        #[borrow]
        Erc20 erc20;
        #[borrow]
        Ownable2Step ownable;
    }
}

#[public]
#[inherit(Erc20, Ownable2Step)]
impl Ownable2StepExample {
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
