#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::{flash_mint, Erc20FlashMint, IErc3156FlashLender},
    Erc20,
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct Erc20FlashMintExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    flash_mint: Erc20FlashMint,
}

#[public]
#[inherit(Erc20)]
impl Erc20FlashMintExample {
    fn max_flash_loan(&self, token: Address) -> U256 {
        self.flash_mint.max_flash_loan(token, &self.erc20)
    }

    fn flash_fee(
        &self,
        token: Address,
        value: U256,
    ) -> Result<U256, flash_mint::Error> {
        Ok(self.flash_mint.flash_fee(token, value)?)
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        value: U256,
        data: Bytes,
    ) -> Result<bool, flash_mint::Error> {
        Ok(self.flash_mint.flash_loan(
            receiver,
            token,
            value,
            data,
            &mut self.erc20,
        )?)
    }

    fn mint(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<(), flash_mint::Error> {
        Ok(self.erc20._mint(to, value)?)
    }
}
