#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::token::erc721::{
    extensions::{wrapper, Erc721Wrapper},
    Erc721,
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
#[inherit(Erc721)]
impl Erc721WrapperExample {
    #[constructor]
    fn constructor(&mut self, underlying_token: Address) {
        self.erc721_wrapper.constructor(underlying_token);
    }

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
