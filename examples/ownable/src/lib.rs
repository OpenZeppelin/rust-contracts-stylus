#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use contracts::{access::ownable::Ownable, erc20::ERC20};
use stylus_sdk::{
    msg,
    prelude::{entrypoint, external, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC20 erc20;
        #[borrow]
        Ownable ownable;
    }
}

#[external]
#[inherit(ERC20, Ownable)]
impl Token {
    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        self.ownable.constructor(msg::sender())?;
        Ok(())
    }

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
