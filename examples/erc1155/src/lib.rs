#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc1155::{
        extensions::{Erc1155MetadataUri, IErc1155Burnable},
        Erc1155,
    },
    utils::introspection::erc165::IErc165,
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

    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.erc1155.burn(account, token_id, value)?;
        Ok(())
    }

    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.erc1155.burn_batch(account, token_ids, values)?;
        Ok(())
    }

    pub fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc1155::supports_interface(interface_id)
            || Erc1155MetadataUri::supports_interface(interface_id)
    }
}
