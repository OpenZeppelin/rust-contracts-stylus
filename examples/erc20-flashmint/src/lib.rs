#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::IERC3156FlashLender, Erc20,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Erc20FlashMintExample {
        #[borrow]
        Erc20 erc20;
    }
}

#[public]
#[inherit(Erc20)]
impl Erc20FlashMintExample {
    fn mint(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc20._mint(to, value)?)
    }

    fn max_flash_loan(&self, token: Address) -> U256 {
        self.erc20.max_flash_loan(token)
    }

    fn flash_fee(&self, token: Address, amount: U256) -> Result<U256, Vec<u8>> {
        self.erc20.flash_fee(token, amount)?;
        Ok(amount.checked_mul(U256::from(1)).unwrap() / U256::from(100))
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: Bytes,
    ) -> Result<bool, Vec<u8>> {
        Ok(self.erc20.flash_loan(receiver, token, value, data)?)
    }
}

impl Erc20FlashMintExample {
    pub fn _flash_fee_receiver(&self) -> Address {
        Address::ZERO
    }
}
