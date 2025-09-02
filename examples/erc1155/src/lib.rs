#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::result_large_err, clippy::needless_pass_by_value)]

extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc1155::{self, extensions::IErc1155Burnable, Erc1155, IErc1155},
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{aliases::B32, Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc1155Example {
    erc1155: Erc1155,
}

#[public]
#[implements(IErc1155<Error = erc1155::Error>, IErc1155Burnable<Error = erc1155::Error>, IErc165)]
impl Erc1155Example {
    fn mint(
        &mut self,
        to: Address,
        token_id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<(), erc1155::Error> {
        self.erc1155._mint(to, token_id, amount, &data)
    }

    fn mint_batch(
        &mut self,
        to: Address,
        token_ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes,
    ) -> Result<(), erc1155::Error> {
        self.erc1155._mint_batch(to, token_ids, amounts, &data)
    }
}

#[public]
impl IErc1155 for Erc1155Example {
    type Error = erc1155::Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.erc1155.balance_of(account, id)
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, Self::Error> {
        self.erc1155.balance_of_batch(accounts, ids)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error> {
        self.erc1155.set_approval_for_all(operator, approved)
    }

    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool {
        self.erc1155.is_approved_for_all(account, operator)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.erc1155.safe_transfer_from(from, to, id, value, data)
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.erc1155.safe_batch_transfer_from(from, to, ids, values, data)
    }
}

#[public]
impl IErc1155Burnable for Erc1155Example {
    type Error = erc1155::Error;

    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Self::Error> {
        self.erc1155.burn(account, token_id, value)
    }

    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Self::Error> {
        self.erc1155.burn_batch(account, token_ids, values)
    }
}

#[public]
impl IErc165 for Erc1155Example {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc1155.supports_interface(interface_id)
    }
}
