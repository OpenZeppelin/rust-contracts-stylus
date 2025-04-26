#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc721::{
        self,
        extensions::{
            enumerable, Erc721Enumerable as Enumerable, IErc721Burnable,
        },
        Erc721, IErc721,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[derive(SolidityError, Debug)]
enum Error {
    OutOfBoundsIndex(enumerable::ERC721OutOfBoundsIndex),
    EnumerableForbiddenBatchMint(
        enumerable::ERC721EnumerableForbiddenBatchMint,
    ),
    InvalidOwner(erc721::ERC721InvalidOwner),
    NonexistentToken(erc721::ERC721NonexistentToken),
    IncorrectOwner(erc721::ERC721IncorrectOwner),
    InvalidSender(erc721::ERC721InvalidSender),
    InvalidReceiver(erc721::ERC721InvalidReceiver),
    InvalidReceiverWithReason(erc721::InvalidReceiverWithReason),
    InsufficientApproval(erc721::ERC721InsufficientApproval),
    InvalidApprover(erc721::ERC721InvalidApprover),
    InvalidOperator(erc721::ERC721InvalidOperator),
}

impl From<enumerable::Error> for Error {
    fn from(value: enumerable::Error) -> Self {
        match value {
            enumerable::Error::OutOfBoundsIndex(e) => {
                Error::OutOfBoundsIndex(e)
            }
            enumerable::Error::EnumerableForbiddenBatchMint(e) => {
                Error::EnumerableForbiddenBatchMint(e)
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
struct Erc721Example {
    #[borrow]
    erc721: Erc721,
    #[borrow]
    enumerable: Enumerable,
}

#[public]
#[inherit(Erc721, Enumerable)]
impl Erc721Example {
    fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        // Retrieve the owner.
        let owner = self.erc721.owner_of(token_id)?;

        self.erc721.burn(token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._remove_token_from_all_tokens_enumeration(token_id);

        Ok(())
    }

    fn mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self.erc721._mint(to, token_id)?;

        // Update the extension's state.
        self.enumerable._add_token_to_all_tokens_enumeration(token_id);
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    fn safe_mint(
        &mut self,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.erc721._safe_mint(to, token_id, &data)?;

        // Update the extension's state.
        self.enumerable._add_token_to_all_tokens_enumeration(token_id);
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.safe_transfer_from(from, to, token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.safe_transfer_from_with_data(from, to, token_id, data)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        // Retrieve the previous owner.
        let previous_owner = self.erc721.owner_of(token_id)?;

        self.erc721.transfer_from(from, to, token_id)?;

        // Update the extension's state.
        self.enumerable._remove_token_from_owner_enumeration(
            previous_owner,
            token_id,
            &self.erc721,
        )?;
        self.enumerable._add_token_to_owner_enumeration(
            to,
            token_id,
            &self.erc721,
        )?;

        Ok(())
    }

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc721::supports_interface(interface_id)
            || Enumerable::supports_interface(interface_id)
    }
}
