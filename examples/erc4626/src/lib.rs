#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc20::{
        extensions::{
            erc4626, Erc20Metadata, Erc4626, IErc20Metadata, IErc4626,
        },
        Erc20, IErc20,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    alloy_primitives::{aliases::B32, Address, U256, U8},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc4626Example {
    erc4626: Erc4626,
    erc20: Erc20,
    metadata: Erc20Metadata,
}

#[public]
#[implements(IErc4626<Error = erc4626::Error>, IErc20<Error = erc4626::Error>, IErc20Metadata, IErc165)]
impl Erc4626Example {
    #[constructor]
    fn constructor(
        &mut self,
        asset: Address,
        decimals_offset: U8,
        name: String,
        symbol: String,
    ) {
        self.erc4626.constructor(asset, decimals_offset);
        self.metadata.constructor(name, symbol);
    }
}

#[public]
impl IErc4626 for Erc4626Example {
    type Error = erc4626::Error;

    fn asset(&self) -> Address {
        self.erc4626.asset()
    }

    fn total_assets(&self) -> Result<U256, Self::Error> {
        self.erc4626.total_assets()
    }

    fn convert_to_shares(&self, assets: U256) -> Result<U256, Self::Error> {
        self.erc4626.convert_to_shares(assets, &self.erc20)
    }

    fn convert_to_assets(&self, shares: U256) -> Result<U256, Self::Error> {
        self.erc4626.convert_to_assets(shares, &self.erc20)
    }

    fn max_deposit(&self, receiver: Address) -> U256 {
        self.erc4626.max_deposit(receiver)
    }

    fn preview_deposit(&self, assets: U256) -> Result<U256, Self::Error> {
        self.erc4626.preview_deposit(assets, &self.erc20)
    }

    fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error> {
        self.erc4626.deposit(assets, receiver, &mut self.erc20)
    }

    fn max_mint(&self, receiver: Address) -> U256 {
        self.erc4626.max_mint(receiver)
    }

    fn preview_mint(&self, shares: U256) -> Result<U256, Self::Error> {
        self.erc4626.preview_mint(shares, &self.erc20)
    }

    fn mint(
        &mut self,
        shares: U256,
        receiver: Address,
    ) -> Result<U256, Self::Error> {
        self.erc4626.mint(shares, receiver, &mut self.erc20)
    }

    fn max_withdraw(&self, owner: Address) -> Result<U256, Self::Error> {
        self.erc4626.max_withdraw(owner, &self.erc20)
    }

    fn preview_withdraw(&self, assets: U256) -> Result<U256, Self::Error> {
        self.erc4626.preview_withdraw(assets, &self.erc20)
    }

    fn withdraw(
        &mut self,
        assets: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Self::Error> {
        self.erc4626.withdraw(assets, receiver, owner, &mut self.erc20)
    }

    fn max_redeem(&self, owner: Address) -> U256 {
        self.erc4626.max_redeem(owner, &self.erc20)
    }

    fn preview_redeem(&self, shares: U256) -> Result<U256, Self::Error> {
        self.erc4626.preview_redeem(shares, &self.erc20)
    }

    fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<U256, Self::Error> {
        self.erc4626.redeem(shares, receiver, owner, &mut self.erc20)
    }
}

#[public]
impl IErc20 for Erc4626Example {
    type Error = erc4626::Error;

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.transfer(to, value)?)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.approve(spender, value)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.transfer_from(from, to, value)?)
    }
}

#[public]
impl IErc20Metadata for Erc4626Example {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn decimals(&self) -> U8 {
        self.erc4626.decimals()
    }
}

#[public]
impl IErc165 for Erc4626Example {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc4626>::interface_id() == interface_id
            || self.erc20.supports_interface(interface_id)
            || self.metadata.supports_interface(interface_id)
    }
}
