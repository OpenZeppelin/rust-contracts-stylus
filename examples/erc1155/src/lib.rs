#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc1155::{
    extensions::Erc1155MetadataUri, Erc1155,
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, sol_storage},
};

sol_storage! {
    #[entrypoint]
    struct Erc1155Example {
        #[borrow]
        Erc1155 erc1155;
        #[borrow]
        Erc1155MetadataUri metadata_uri;
    }
}

#[public]
#[inherit(Erc1155, Erc1155MetadataUri)]
impl Erc1155Example {
    pub fn mint(
        &mut self,
        to: Address,
        token_id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.erc1155._mint(to, token_id, amount, &data)?;
        Ok(())
    }

    pub fn mint_batch(
        &mut self,
        to: Address,
        token_ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.erc1155._mint_batch(to, token_ids, amounts, &data)?;
        Ok(())
    }
}
