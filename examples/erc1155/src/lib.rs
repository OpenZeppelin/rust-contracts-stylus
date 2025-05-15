#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc1155::{
    self, extensions::IErc1155Burnable, Erc1155,
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[derive(SolidityError, Debug)]
enum Error {
    InsufficientBalance(erc1155::ERC1155InsufficientBalance),
    InvalidSender(erc1155::ERC1155InvalidSender),
    InvalidReceiver(erc1155::ERC1155InvalidReceiver),
    InvalidReceiverWithReason(erc1155::InvalidReceiverWithReason),
    MissingApprovalForAll(erc1155::ERC1155MissingApprovalForAll),
    InvalidApprover(erc1155::ERC1155InvalidApprover),
    InvalidOperator(erc1155::ERC1155InvalidOperator),
    InvalidArrayLength(erc1155::ERC1155InvalidArrayLength),
}

impl From<erc1155::Error> for Error {
    fn from(value: erc1155::Error) -> Self {
        match value {
            erc1155::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc1155::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc1155::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc1155::Error::InvalidReceiverWithReason(e) => {
                Error::InvalidReceiverWithReason(e)
            }
            erc1155::Error::MissingApprovalForAll(e) => {
                Error::MissingApprovalForAll(e)
            }
            erc1155::Error::InvalidApprover(e) => Error::InvalidApprover(e),
            erc1155::Error::InvalidOperator(e) => Error::InvalidOperator(e),
            erc1155::Error::InvalidArrayLength(e) => {
                Error::InvalidArrayLength(e)
            }
        }
    }
}

#[entrypoint]
#[storage]
struct Erc1155Example {
    #[borrow]
    erc1155: Erc1155,
}

#[public]
#[inherit(Erc1155)]
impl Erc1155Example {
    fn mint(
        &mut self,
        to: Address,
        token_id: U256,
        amount: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.erc1155._mint(to, token_id, amount, &data)?;
        Ok(())
    }

    fn mint_batch(
        &mut self,
        to: Address,
        token_ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        self.erc1155._mint_batch(to, token_ids, amounts, &data)?;
        Ok(())
    }

    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Error> {
        self.erc1155.burn(account, token_id, value)?;
        Ok(())
    }

    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        self.erc1155.burn_batch(account, token_ids, values)?;
        Ok(())
    }
}
