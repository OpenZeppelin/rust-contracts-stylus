#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::extensions::{
    Erc20FlashMint, IErc3156FlashLender,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, storage},
};

#[entrypoint]
#[storage]
struct Erc20FlashMintExample {
    #[borrow]
    erc20_flash_mint: Erc20FlashMint,
}

#[public]
#[inherit(Erc20FlashMint)]
impl Erc20FlashMintExample {
    fn max_flash_loan(&self, token: Address) -> U256 {
        self.erc20_flash_mint.max_flash_loan(token)
    }

    fn flash_fee(&self, token: Address, amount: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc20_flash_mint.flash_fee(token, amount)?)
    }

    fn flash_loan(
        &mut self,
        receiver: Address,
        token: Address,
        amount: U256,
        data: Bytes,
    ) -> Result<bool, Vec<u8>> {
        Ok(self.erc20_flash_mint.flash_loan(receiver, token, amount, data)?)
    }

    fn mint(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc20_flash_mint._mint(to, value)?)
    }
}
