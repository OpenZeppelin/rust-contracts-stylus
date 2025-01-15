#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        extensions::{Erc721Enumerable as Enumerable, IErc721Burnable},
        Erc721, IErc721,
    },
    utils::{introspection::erc165::IErc165, Pausable},
};
use stylus_sdk::{
    abi::Bytes,
    prelude::{entrypoint, public, storage},
};

#[entrypoint]
#[storage]
struct Erc721Example {
    #[borrow]
    pub erc721: Erc721,
    #[borrow]
    pub enumerable: Enumerable,
    #[borrow]
    pub pausable: Pausable,
}

#[public]
#[inherit(Erc721, Enumerable, Pausable)]
impl Erc721Example {
    pub fn burn(&mut self, token_id: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        // Retrieve the owner.
        let owner = self.erc721.owner_of(token_id)?;

        self.erc721.burn(token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._remove_token_from_all_tokens_enumeration(token_id);

        Ok(())
    }

    pub fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc721._mint(to, token_id)?;

        // Update the extension's state.
        self.enumerable._add_token_to_all_tokens_enumeration(token_id);
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    pub fn safe_mint(
        &mut self,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        self.erc721._safe_mint(to, token_id, &data)?;

        // Update the extension's state.
        self.enumerable._add_token_to_all_tokens_enumeration(token_id);
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.safe_transfer_from(from, to, token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.safe_transfer_from_with_data(from, to, token_id, data)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;

        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.transfer_from(from, to, token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    pub fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(interface_id)
            || Enumerable::supports_interface(interface_id)
    }

    /// WARNING: These functions are intended for **testing purposes** only. In
    /// **production**, ensure strict access control to prevent unauthorized
    /// pausing or unpausing, which can disrupt contract functionality. Remove
    /// or secure these functions before deployment.
    pub fn pause(&mut self) -> Result<(), Vec<u8>> {
        self.pausable.pause().map_err(|e| e.into())
    }

    pub fn unpause(&mut self) -> Result<(), Vec<u8>> {
        self.pausable.unpause().map_err(|e| e.into())
    }
}
