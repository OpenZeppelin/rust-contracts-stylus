#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc1155::{
        extensions::{Erc1155MetadataUri, IErc1155Burnable},
        Erc1155, IErc1155,
    },
    utils::{introspection::erc165::IErc165, Pausable},
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
        #[borrow]
        Pausable pausable;
    }
}

#[public]
#[inherit(Erc1155, Erc1155MetadataUri, Pausable)]
impl Erc1155Example {
    pub fn mint(
        &mut self,
        to: Address,
        token_id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
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
        self.pausable.when_not_paused()?;
        self.erc1155._mint_batch(to, token_ids, amounts, &data)?;
        Ok(())
    }

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

    pub fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc1155.burn(account, token_id, value)?;
        Ok(())
    }

    pub fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc1155.burn_batch(account, token_ids, values)?;
        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc1155.safe_transfer_from(from, to, id, value, data)?;
        Ok(())
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc1155.safe_batch_transfer_from(from, to, ids, values, data)?;
        Ok(())
    }

    pub fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc1155::supports_interface(interface_id)
            || Erc1155MetadataUri::supports_interface(interface_id)
    }
}
