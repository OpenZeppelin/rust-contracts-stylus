#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{extensions::Erc20Flashmint, IErc20};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc20FlashmintExample {
        #[borrow]
        Erc20Flashmint erc20_flashmint;
    }
}

#[public]
#[inherit(Erc20Flashmint)]
impl Erc20FlashmintExample {
    pub fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.erc20_flashmint.erc20.transfer(to, value).map_err(|e| e.into())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.erc20_flashmint
            .erc20
            .transfer_from(from, to, value)
            .map_err(|e| e.into())
    }

    // Add token minting feature.
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc20_flashmint.erc20._mint(account, value)?;
        Ok(())
    }
}
