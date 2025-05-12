#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::{aliases::U96, Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        self,
        extensions::{
            consecutive::{self, Erc721Consecutive},
            IErc721Burnable,
        },
        IErc721,
    },
    utils::{introspection::erc165::IErc165, structs::checkpoints},
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[derive(SolidityError, Debug)]
enum Error {
    InvalidOwner(erc721::ERC721InvalidOwner),
    NonexistentToken(erc721::ERC721NonexistentToken),
    IncorrectOwner(erc721::ERC721IncorrectOwner),
    InvalidSender(erc721::ERC721InvalidSender),
    InvalidReceiver(erc721::ERC721InvalidReceiver),
    InvalidReceiverWithReason(erc721::InvalidReceiverWithReason),
    InsufficientApproval(erc721::ERC721InsufficientApproval),
    InvalidApprover(erc721::ERC721InvalidApprover),
    InvalidOperator(erc721::ERC721InvalidOperator),
    CheckpointUnorderedInsertion(checkpoints::CheckpointUnorderedInsertion),
    ForbiddenBatchMint(consecutive::ERC721ForbiddenBatchMint),
    ExceededMaxBatchMint(consecutive::ERC721ExceededMaxBatchMint),
    ForbiddenMint(consecutive::ERC721ForbiddenMint),
    ForbiddenBatchBurn(consecutive::ERC721ForbiddenBatchBurn),
}

impl From<consecutive::Error> for Error {
    fn from(value: consecutive::Error) -> Self {
        match value {
            consecutive::Error::InvalidOwner(e) => Error::InvalidOwner(e),
            consecutive::Error::NonexistentToken(e) => {
                Error::NonexistentToken(e)
            }
            consecutive::Error::IncorrectOwner(e) => Error::IncorrectOwner(e),
            consecutive::Error::InvalidSender(e) => Error::InvalidSender(e),
            consecutive::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            consecutive::Error::InvalidReceiverWithReason(e) => {
                Error::InvalidReceiverWithReason(e)
            }
            consecutive::Error::InsufficientApproval(e) => {
                Error::InsufficientApproval(e)
            }
            consecutive::Error::InvalidApprover(e) => Error::InvalidApprover(e),
            consecutive::Error::InvalidOperator(e) => Error::InvalidOperator(e),
            consecutive::Error::ForbiddenBatchMint(e) => {
                Error::ForbiddenBatchMint(e)
            }
            consecutive::Error::ExceededMaxBatchMint(e) => {
                Error::ExceededMaxBatchMint(e)
            }
            consecutive::Error::ForbiddenMint(e) => Error::ForbiddenMint(e),
            consecutive::Error::ForbiddenBatchBurn(e) => {
                Error::ForbiddenBatchBurn(e)
            }
            consecutive::Error::CheckpointUnorderedInsertion(e) => {
                Error::CheckpointUnorderedInsertion(e)
            }
        }
    }
}

impl From<erc721::Error> for Error {
    fn from(value: erc721::Error) -> Self {
        match value {
            erc721::Error::InvalidOwner(e) => Error::InvalidOwner(e),
            erc721::Error::NonexistentToken(e) => Error::NonexistentToken(e),
            erc721::Error::IncorrectOwner(e) => Error::IncorrectOwner(e),
            erc721::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc721::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc721::Error::InvalidReceiverWithReason(e) => {
                Error::InvalidReceiverWithReason(e)
            }
            erc721::Error::InsufficientApproval(e) => {
                Error::InsufficientApproval(e)
            }
            erc721::Error::InvalidApprover(e) => Error::InvalidApprover(e),
            erc721::Error::InvalidOperator(e) => Error::InvalidOperator(e),
        }
    }
}

#[entrypoint]
#[storage]
struct Erc721ConsecutiveExample {
    #[borrow]
    erc721: Erc721Consecutive,
}

#[public]
#[implements(IErc721<Error=Error>, IErc721Burnable<Error=Error>, IErc165)]
impl Erc721ConsecutiveExample {
    #[constructor]
    fn constructor(
        &mut self,
        receivers: Vec<Address>,
        amounts: Vec<U96>,
        first_consecutive_id: U96,
        max_batch_size: U96,
    ) -> Result<(), Error> {
        self.erc721.first_consecutive_id.set(first_consecutive_id);
        self.erc721.max_batch_size.set(max_batch_size);
        for (&receiver, &amount) in receivers.iter().zip(amounts.iter()) {
            self.erc721._mint_consecutive(receiver, amount)?;
        }
        Ok(())
    }

    fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        Ok(self.erc721._mint(to, token_id)?)
    }
}

#[public]
impl IErc721 for Erc721ConsecutiveExample {
    type Error = Error;

    fn balance_of(&self, owner: Address) -> Result<U256, Error> {
        Ok(self.erc721.balance_of(owner)?)
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, Error> {
        Ok(self.erc721.owner_of(token_id)?)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        Ok(self.erc721.safe_transfer_from(from, to, token_id)?)
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        Ok(self
            .erc721
            .safe_transfer_from_with_data(from, to, token_id, data)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        Ok(self.erc721.transfer_from(from, to, token_id)?)
    }

    fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        Ok(self.erc721.approve(to, token_id)?)
    }

    fn set_approval_for_all(
        &mut self,
        to: Address,
        approved: bool,
    ) -> Result<(), Error> {
        Ok(self.erc721.set_approval_for_all(to, approved)?)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, Error> {
        Ok(self.erc721.get_approved(token_id)?)
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721.is_approved_for_all(owner, operator)
    }
}

#[public]
impl IErc721Burnable for Erc721ConsecutiveExample {
    type Error = Error;

    fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        Ok(self.erc721._burn(token_id)?)
    }
}

#[public]
impl IErc165 for Erc721ConsecutiveExample {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.erc721.supports_interface(interface_id)
    }
}
