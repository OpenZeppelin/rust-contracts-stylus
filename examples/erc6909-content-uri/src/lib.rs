#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::result_large_err)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus::{
    token::erc6909::{
        self,
        extensions::{Erc6909ContentUri, IErc6909ContentUri},
        IErc6909,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc6909ContentUriExample {
    erc6909_content_uri: Erc6909ContentUri,
}

#[public]
#[implements(IErc6909<Error = erc6909::Error>, IErc6909ContentUri, IErc165)]
impl Erc6909ContentUriExample {
    fn mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909_content_uri._mint(to, id, amount)
    }

    fn burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909_content_uri._burn(from, id, amount)
    }

    fn set_contract_uri(&mut self, new_contract_uri: String) {
        self.erc6909_content_uri._set_contract_uri(new_contract_uri);
    }

    fn set_token_uri(&mut self, id: U256, new_token_uri: String) {
        self.erc6909_content_uri._set_token_uri(id, new_token_uri);
    }
}

#[public]
impl IErc6909 for Erc6909ContentUriExample {
    type Error = erc6909::Error;

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909_content_uri.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909_content_uri.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909_content_uri.is_operator(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_content_uri.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909_content_uri.set_operator(spender, approved)
    }

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_content_uri.transfer(receiver, id, amount)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909_content_uri.transfer_from(sender, receiver, id, amount)
    }
}

#[public]
impl IErc6909ContentUri for Erc6909ContentUriExample {
    fn contract_uri(&self) -> String {
        self.erc6909_content_uri.contract_uri()
    }

    fn token_uri(&self, id: U256) -> String {
        self.erc6909_content_uri.token_uri(id)
    }
}

#[public]
impl IErc165 for Erc6909ContentUriExample {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc6909_content_uri.supports_interface(interface_id)
    }
}
