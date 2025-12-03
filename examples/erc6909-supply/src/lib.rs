#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::result_large_err)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus::{
    token::erc6909::{
        self,
        extensions::{Erc6909TokenSupply, IErc6909TokenSupply},
        IErc6909,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc6909TokenSupplyExample {
    erc6909_token_supply: Erc6909TokenSupply,
}

#[public]
#[implements(IErc6909<Error = erc6909::Error>, IErc6909TokenSupply, IErc165)]
impl Erc6909TokenSupplyExample {
    fn mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909_token_supply._mint(to, id, amount)
    }

    fn burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909_token_supply._burn(from, id, amount)
    }
}

#[public]
impl IErc6909 for Erc6909TokenSupplyExample {
    type Error = erc6909::Error;

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909_token_supply.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909_token_supply.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909_token_supply.is_operator(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_token_supply.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909_token_supply.set_operator(spender, approved)
    }

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_token_supply.transfer(receiver, id, amount)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_token_supply.transfer_from(sender, receiver, id, amount)
    }
}

#[public]
impl IErc6909TokenSupply for Erc6909TokenSupplyExample {
    fn total_supply(&self, id: U256) -> U256 {
        self.erc6909_token_supply.total_supply(id)
    }
}

#[public]
impl IErc165 for Erc6909TokenSupplyExample {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc6909_token_supply.supports_interface(interface_id)
    }
}
