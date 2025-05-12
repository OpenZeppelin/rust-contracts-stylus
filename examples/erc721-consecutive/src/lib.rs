#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::FixedBytes;
use alloy_primitives::{aliases::U96, Address, /* FixedBytes, */ U256};
use openzeppelin_stylus::{
    token::erc721::{
        extensions::{consecutive, Erc721Consecutive, IErc721Burnable},
        IErc721,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{abi::Bytes, prelude::*};
#[entrypoint]
#[storage]
struct Erc721ConsecutiveExample {
    #[borrow]
    erc721_consecutive: Erc721Consecutive,
}

#[public]
#[implements(IErc721<Error=consecutive::Error>, IErc721Burnable<Error=consecutive::Error>, IErc165)]
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

    fn mint(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), consecutive::Error> {
        self.erc721_consecutive._mint(to, token_id)
    }
}

#[public]
impl IErc721 for Erc721ConsecutiveExample {
    type Error = consecutive::Error;

    fn balance_of(&self, owner: Address) -> Result<U256, consecutive::Error> {
        Ok(self.erc721_consecutive.balance_of(owner)?)
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, consecutive::Error> {
        Ok(self.erc721_consecutive.owner_of(token_id)?)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), consecutive::Error> {
        Ok(self.erc721_consecutive.safe_transfer_from(from, to, token_id)?)
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), consecutive::Error> {
        Ok(self
            .erc721_consecutive
            .safe_transfer_from_with_data(from, to, token_id, data)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), consecutive::Error> {
        Ok(self.erc721_consecutive.transfer_from(from, to, token_id)?)
    }

    fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), consecutive::Error> {
        Ok(self.erc721_consecutive.approve(to, token_id)?)
    }

    fn set_approval_for_all(
        &mut self,
        to: Address,
        approved: bool,
    ) -> Result<(), consecutive::Error> {
        Ok(self.erc721_consecutive.set_approval_for_all(to, approved)?)
    }

    fn get_approved(
        &self,
        token_id: U256,
    ) -> Result<Address, consecutive::Error> {
        Ok(self.erc721_consecutive.get_approved(token_id)?)
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721_consecutive.is_approved_for_all(owner, operator)
    }
}

#[public]
impl IErc721Burnable for Erc721ConsecutiveExample {
    type Error = consecutive::Error;

    fn burn(&mut self, token_id: U256) -> Result<(), consecutive::Error> {
        self.erc721_consecutive._burn(token_id)
    }
}

#[public]
impl IErc165 for Erc721ConsecutiveExample {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.erc721_consecutive.supports_interface(interface_id)
    }
}
