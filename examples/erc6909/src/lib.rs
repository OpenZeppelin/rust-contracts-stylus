#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::result_large_err)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc6909::{self, Erc6909, IErc6909},
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    alloy_primitives::{aliases::B32, Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc6909Example {
    erc6909: Erc6909,
}

#[public]
#[implements(IErc6909<Error = erc6909::Error>, IErc165)]
impl Erc6909Example {
    fn mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909._mint(to, id, amount)
    }

    fn burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909._burn(from, id, amount)
    }
}

#[public]
impl IErc6909 for Erc6909Example {
    type Error = erc6909::Error;

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909.is_operator(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909.set_operator(spender, approved)
    }

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.transfer(receiver, id, amount)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.transfer_from(sender, receiver, id, amount)
    }
}

#[public]
impl IErc165 for Erc6909Example {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc6909.supports_interface(interface_id)
    }
}
