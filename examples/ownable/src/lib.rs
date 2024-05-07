#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use contracts::{
    access::ownable::{Error, Ownable},
    erc20::ERC20,
};
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
impl Token {
    pub fn constructor(&mut self) -> Result<(), Error> {
        self.ownable.constructor(msg::sender())
    }

    pub fn transfer(&mut self, to: Address, value: U256) {
        self.ownable.only_owner().expect("caller doesn't own the contract");
        self.erc20
            .transfer(to, value)
            .expect("recipient should not be Address::ZERO");
    }
}
