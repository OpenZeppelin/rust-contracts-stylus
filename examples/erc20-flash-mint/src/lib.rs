#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::{Erc20FlashMint, IErc3156FlashLender},
    Erc20,
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
        #[borrow]
        Erc20FlashMint erc20_flash_mint;
    }
}

#[public]
#[inherit(Erc20)]
impl Erc20FlashMintExample {
    fn max_flash_loan(&self, token: Address) -> U256 {
        self.erc20_flash_mint.max_flash_loan(token, &self.erc20)
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
        Ok(self.erc20_flash_mint.flash_loan(
            receiver,
            token,
            amount,
            data,
            &mut self.erc20,
        )?)
    }

    fn mint(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        Ok(self.erc20._mint(to, value)?)
    }
}
