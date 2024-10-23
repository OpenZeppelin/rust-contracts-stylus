#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::ownable::Ownable,
    token::erc20::{Erc20, IErc20},
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct OwnableExample {
        #[borrow]
        Erc20 erc20;
        #[borrow]
        Ownable ownable;
    }
}

#[public]
#[inherit(Erc20, Ownable)]
impl OwnableExample {
    pub fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;
        self.erc20.transfer(to, value)?;
        Ok(())
    }
}
