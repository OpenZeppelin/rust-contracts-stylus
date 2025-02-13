#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc1155::{extensions::IErc1155Burnable, Erc1155, IErc1155},
    utils::{introspection::erc165::IErc165, Pausable},
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct Erc1155Example {
    #[borrow]
    pub erc1155: Erc1155,
    #[borrow]
    pub pausable: Pausable,
}

#[public]
#[inherit(Erc1155, Pausable)]
impl Erc1155Example {
    fn mint(
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

    fn mint_batch(
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

    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc1155.burn(account, token_id, value)?;
        Ok(())
    }

    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc1155.burn_batch(account, token_ids, values)?;
        Ok(())
    }

    fn safe_transfer_from(
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

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc1155::supports_interface(interface_id)
    }

    /// WARNING: These functions are intended for **testing purposes** only. In
    /// **production**, ensure strict access control to prevent unauthorized
    /// pausing or unpausing, which can disrupt contract functionality. Remove
    /// or secure these functions before deployment.
    fn pause(&mut self) -> Result<(), Vec<u8>> {
        self.pausable.pause().map_err(|e| e.into())
    }

    fn unpause(&mut self) -> Result<(), Vec<u8>> {
        self.pausable.unpause().map_err(|e| e.into())
    }
}
