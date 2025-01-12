#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::{
    extensions::{Erc20Metadata, Erc4626, IErc20Metadata, IERC4626},
    utils::SafeErc20,
    Erc20,
};
use stylus_sdk::{
    contract,
    prelude::{entrypoint, public, storage},
};

const DECIMALS: u8 = 18;

#[entrypoint]
#[storage]
struct Erc4626Example {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub safe_erc20: SafeErc20,
    #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub erc4626: Erc4626,
}

#[public]
#[inherit(Erc20)]
impl Erc4626Example {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn decimals(&self) -> u8 {
        DECIMALS
    }

    fn asset(&self) -> Address {
        self.erc4626.asset()
    }

    fn total_assets(&self) -> U256 {
        self.erc4626.total_assets(&self.erc20)
    }

    fn convert_to_shares(&mut self, assets: U256) -> U256 {
        self.erc4626.convert_to_shares(assets, &mut self.erc20)
    }

    fn convert_to_assets(&mut self, shares: U256) -> U256 {
        self.erc4626.convert_to_assets(shares, &mut self.erc20)
    }

    fn preview_deposit(&mut self, assets: U256) -> U256 {
        self.erc4626.preview_deposit(assets, &mut self.erc20)
    }

    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.deposit(assets, receiver, &mut self.erc20)?)
    }

    fn preview_mint(&mut self, shares: U256) -> U256 {
        self.erc4626.preview_mint(shares, &mut self.erc20)
    }

    fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.mint(shares, receiver, &mut self.erc20)?)
    }

    fn max_withdraw(&mut self, owner: Address) -> U256 {
        self.erc4626.max_withdraw(owner, &mut self.erc20)
    }

    fn preview_withdraw(&mut self, assets: U256) -> U256 {
        self.erc4626.preview_withdraw(assets, &mut self.erc20)
    }

    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.withdraw(
            assets,
            receiver,
            owner,
            &mut self.erc20,
            &mut self.safe_erc20,
        )?)
    }

    fn max_redeem(&mut self, owner: Address) -> U256 {
        self.erc4626.max_redeem(owner, &mut self.erc20)
    }

    fn preview_redeem(&mut self, shares: U256) -> U256 {
        self.erc4626.preview_redeem(shares, &mut self.erc20)
    }

    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        Ok(self.erc4626.redeem(
            shares,
            receiver,
            owner,
            &mut self.erc20,
            &mut self.safe_erc20,
        )?)
    }
}
