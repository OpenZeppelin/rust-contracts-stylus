#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc1155::extensions::Erc1155Supply;
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Erc1155SupplyExample {
        #[borrow]
        Erc1155Supply erc1155_supply;
    }
}

#[public]
#[inherit(Erc1155Supply)]
impl Erc1155SupplyExample {
    pub fn mint(
        &mut self,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.erc1155_supply._update(Address::ZERO, to, token_ids, values)?;
        Ok(())
    }

    pub fn total_supply(&self, token_id: U256) -> U256 {
        self.erc1155_supply.total_supply(token_id)
    }

    pub fn total_supply_all(&self) -> U256 {
        self.erc1155_supply.total_supply_all()
    }

    pub fn exists(&self, token_id: U256) -> bool {
        self.erc1155_supply.exists(token_id)
    }
}
