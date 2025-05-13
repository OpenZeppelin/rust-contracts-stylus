#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        self,
        extensions::{
            wrapper, wrapper::IErc721Wrapper, Erc721Wrapper, IErc721Burnable,
        },
        Erc721, IErc721,
    },
    utils::introspection::erc165::{Erc165, IErc165},
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct Erc721WrapperExample {
    #[borrow]
    erc721: Erc721,
    #[borrow]
    erc721_wrapper: Erc721Wrapper,
}

#[public]
#[implements(IErc721<Error=erc721::Error>, IErc721Burnable<Error=erc721::Error>, IErc721Wrapper<Error=wrapper::Error>, IErc165)]
impl Erc721WrapperExample {
    #[constructor]
    fn constructor(&mut self, underlying_token: Address) {
        self.erc721_wrapper.constructor(underlying_token);
    }
}

#[public]
impl IErc721 for Erc721WrapperExample {
    type Error = erc721::Error;

    fn balance_of(&self, owner: Address) -> Result<U256, erc721::Error> {
        self.erc721.balance_of(owner)
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, erc721::Error> {
        self.erc721.owner_of(token_id)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), erc721::Error> {
        self.erc721.safe_transfer_from(from, to, token_id)
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), erc721::Error> {
        self.erc721.safe_transfer_from_with_data(from, to, token_id, data)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), erc721::Error> {
        self.erc721.transfer_from(from, to, token_id)
    }

    fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), erc721::Error> {
        self.erc721.approve(to, token_id)
    }

    fn set_approval_for_all(
        &mut self,
        to: Address,
        approved: bool,
    ) -> Result<(), erc721::Error> {
        self.erc721.set_approval_for_all(to, approved)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, erc721::Error> {
        self.erc721.get_approved(token_id)
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721.is_approved_for_all(owner, operator)
    }
}

#[public]
impl IErc721Burnable for Erc721WrapperExample {
    type Error = erc721::Error;

    fn burn(&mut self, token_id: U256) -> Result<(), erc721::Error> {
        self.erc721._burn(token_id)
    }
}

#[public]
impl IErc721Wrapper for Erc721WrapperExample {
    type Error = wrapper::Error;

    fn underlying(&self) -> Address {
        self.erc721_wrapper.underlying()
    }

    fn deposit_for(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
    ) -> Result<bool, wrapper::Error> {
        self.erc721_wrapper.deposit_for(account, token_ids, &mut self.erc721)
    }

    fn withdraw_to(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
    ) -> Result<bool, wrapper::Error> {
        self.erc721_wrapper.withdraw_to(account, token_ids, &mut self.erc721)
    }

    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<FixedBytes<4>, wrapper::Error> {
        self.erc721_wrapper.on_erc721_received(
            operator,
            from,
            token_id,
            &data,
            &mut self.erc721,
        )
    }
}

#[public]
impl IErc165 for Erc721WrapperExample {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(&self.erc721, interface_id)
            || Erc165::interface_id() == interface_id
    }
}
