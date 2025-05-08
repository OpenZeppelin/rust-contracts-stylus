#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::{aliases::U96, Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        extensions::{consecutive, Erc721Consecutive},
        Erc721,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc721ConsecutiveExample {
    #[borrow]
    erc721_consecutive: Erc721Consecutive,
}

#[public]
#[inherit(Erc721Consecutive)]
impl Erc721ConsecutiveExample {
    #[constructor]
    fn constructor(
        &mut self,
        receivers: Vec<Address>,
        amounts: Vec<U96>,
        first_consecutive_id: U96,
        max_batch_size: U96,
    ) -> Result<(), consecutive::Error> {
        self.erc721_consecutive.first_consecutive_id.set(first_consecutive_id);
        self.erc721_consecutive.max_batch_size.set(max_batch_size);
        for (&receiver, &amount) in receivers.iter().zip(amounts.iter()) {
            self.erc721_consecutive._mint_consecutive(receiver, amount)?;
        }
        Ok(())
    }

    fn burn(&mut self, token_id: U256) -> Result<(), consecutive::Error> {
        self.erc721_consecutive._burn(token_id)
    }

    fn mint(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), consecutive::Error> {
        self.erc721_consecutive._mint(to, token_id)
    }

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(interface_id)
    }
}
