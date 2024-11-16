#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{
            capped, flashmint::IERC3156FlashLender, Capped, Erc20Metadata,
            IErc20Burnable,
        Erc20FlashMint,
        },
        Erc20, IErc20,
    },
    utils::{introspection::erc165::IErc165, Pausable},
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

const DECIMALS: u8 = 10;

sol_storage! {
    #[entrypoint]
    struct Erc20FlashmintExample {
        #[borrow]
        Erc20FlashMint erc20_flashmint;
    }
}

#[public]
#[inherit(Erc20FlashMint)]
impl Erc20FlashmintExample {
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
