#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc1155::{
        self,
        extensions::{Erc1155Supply, IErc1155Supply},
        Erc1155, IErc1155,
    },
    utils::introspection::erc165::IErc165,
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
    erc1155_supply: Erc1155Supply,
}

#[public]
#[implements(IErc1155<Error = Error>, IErc1155Supply, IErc165)]
impl Erc1155Example {
    // Add token minting feature.
    fn mint(
        &mut self,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), erc1155::Error> {
        self.erc1155_supply._mint(to, id, value, &data)
    }

    fn mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), erc1155::Error> {
        self.erc1155_supply._mint_batch(to, ids, values, &data)
    }

    // Add token burning feature.
    fn burn(
        &mut self,
        from: Address,
        id: U256,
        value: U256,
    ) -> Result<(), erc1155::Error> {
        self.erc1155_supply._burn(from, id, value)
    }

    fn burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), erc1155::Error> {
        self.erc1155_supply._burn_batch(from, ids, values)
    }
}

#[public]
impl IErc165 for Erc1155Example {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        self.erc1155_supply.supports_interface(interface_id)
    }
}

#[public]
impl IErc1155 for Erc1155Example {
    type Error = Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.erc1155_supply.balance_of(account, id)
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, <Self as IErc1155>::Error> {
        Ok(self.erc1155_supply.balance_of_batch(accounts, ids)?)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), <Self as IErc1155>::Error> {
        Ok(self.erc1155_supply.set_approval_for_all(operator, approved)?)
    }

    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool {
        self.erc1155_supply.is_approved_for_all(account, operator)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), <Self as IErc1155>::Error> {
        Ok(self.erc1155_supply.safe_transfer_from(from, to, id, value, data)?)
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), <Self as IErc1155>::Error> {
        Ok(self
            .erc1155_supply
            .safe_batch_transfer_from(from, to, ids, values, data)?)
    }
}

#[public]
impl IErc1155Supply for Erc1155Example {
    fn total_supply(&self, id: U256) -> U256 {
        self.erc1155_supply.total_supply(id)
    }

    #[selector(name = "totalSupply")]
    fn total_supply_all(&self) -> U256 {
        self.erc1155_supply.total_supply_all()
    }

    fn exists(&self, id: U256) -> bool {
        self.erc1155_supply.exists(id)
    }
}
