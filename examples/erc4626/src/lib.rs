#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256, U8};
use openzeppelin_stylus::token::erc20::{
    extensions::{Erc4626, IErc4626},
    Erc20,
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct Erc4626Example {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub erc4626: Erc4626,
}

#[public]
#[inherit(Erc20)]
impl Erc4626Example {
    fn decimals(&self) -> U8 {
        self.erc4626.decimals()
    }

    fn asset(&self) -> Address {
        self.erc4626.asset()
    }

    fn total_assets(&mut self) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.total_assets()?)
    }

    fn convert_to_shares(&mut self, assets: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.convert_to_shares(assets)?)
    }

    fn convert_to_assets(&mut self, shares: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.convert_to_assets(shares)?)
    }

    fn max_deposit(&self, receiver: Address) -> U256 {
        self.erc4626.max_deposit(receiver)
    }

    fn preview_deposit(&mut self, assets: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.preview_deposit(assets)?)
    }

    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.deposit(assets, receiver, &mut self.erc20)?)
    }

    fn max_mint(&self, receiver: Address) -> U256 {
        self.erc4626.max_mint(receiver)
    }

    fn preview_mint(&mut self, shares: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.preview_mint(shares)?)
    }

    fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.mint(shares, receiver, &mut self.erc20)?)
    }

    fn max_withdraw(&mut self, owner: Address) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.max_withdraw(owner, &self.erc20)?)
    }

    fn preview_withdraw(&mut self, assets: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.preview_withdraw(assets)?)
    }

    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.withdraw(assets, receiver, owner, &mut self.erc20)?)
    }

    fn max_redeem(&self, owner: Address) -> U256 {
        self.erc4626.max_redeem(owner, &self.erc20)
    }

    fn preview_redeem(&mut self, shares: U256) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.preview_redeem(shares)?)
    }

    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.redeem(shares, receiver, owner, &mut self.erc20)?)
    }
}
