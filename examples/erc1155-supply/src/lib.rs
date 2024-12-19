#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc1155::extensions::{
    Erc1155Supply, IErc1155Supply,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, storage},
};

#[entrypoint]
#[storage]
struct Erc1155Example {
    #[borrow]
    pub erc1155_supply: Erc1155Supply,
}

#[public]
#[inherit(Erc1155Supply)]
impl Erc1155Example {
    fn total_supply(&self, id: U256) -> U256 {
        self.erc1155_supply.total_supply(id)
    }

    #[selector(name = "totalSupply")]
    fn total_supply_all(&self) -> U256 {
        self.erc1155_supply.total_supply_all()
    }

    fn exists(&self, id: U256) -> bool {
        self.erc1155_supply.exists(id)
    }

    // Add token minting feature.
    pub fn mint(
        &mut self,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.erc1155_supply._mint(to, id, value, &data)?;
        Ok(())
    }

    pub fn mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.erc1155_supply._mint_batch(to, ids, values, &data)?;
        Ok(())
    }

    // Add token burning feature.
    pub fn burn(
        &mut self,
        from: Address,
        id: U256,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc1155_supply._burn(from, id, value)?;
        Ok(())
    }

    pub fn burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.erc1155_supply._burn_batch(from, ids, values)?;
        Ok(())
    }
}
