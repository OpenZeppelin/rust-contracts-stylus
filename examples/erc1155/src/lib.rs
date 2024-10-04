#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    token::erc1155::{Erc1155, IErc1155},
    utils::Pausable,
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
        Pausable pausable;
    }
}

#[public]
#[inherit(Erc1155, Pausable)]
impl Erc1155Example {
    pub fn set_operator_approvals(
        &mut self,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), Vec<u8>> {
        self.erc1155
            ._operator_approvals
            .setter(owner)
            .setter(operator)
            .set(approved);
        Ok(())
    }

    pub fn mint(
        &mut self,
        to: Address,
        token_id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc1155._mint(to, token_id, amount, data)?;
        Ok(())
    }

    pub fn mint_batch(
        &mut self,
        to: Address,
        token_ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc1155._mint_batch(to, token_ids, amounts, data)?;
        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc1155.safe_transfer_from(from, to, token_id, amount, data)?;
        Ok(())
    }

    pub fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc1155
            .safe_batch_transfer_from(from, to, token_ids, amounts, data)?;
        Ok(())
    }
}
