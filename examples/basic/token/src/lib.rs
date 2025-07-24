#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc20::{
        self,
        extensions::{Erc20Metadata, IErc20Metadata},
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
struct Erc20Example {
    erc20: Erc20,
    metadata: Erc20Metadata,
}

#[public]
#[implements(IErc20<Error = erc20::Error>, IErc20Metadata, IErc165)]
impl Erc20Example {
    #[constructor]
    fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }

    fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), erc20::Error> {
        self.erc20._mint(account, value)
    }
}

#[public]
impl IErc20 for Erc20Example {
    type Error = erc20::Error;

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
        self.erc20.transfer(to, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.approve(spender, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.erc20.transfer_from(from, to, value)
    }
}

#[public]
impl IErc20Metadata for Erc20Example {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    fn decimals(&self) -> U8 {
        self.metadata.decimals()
    }
}

#[public]
impl IErc165 for Erc20Example {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc20.supports_interface(interface_id)
            || self.metadata.supports_interface(interface_id)
    }
}
