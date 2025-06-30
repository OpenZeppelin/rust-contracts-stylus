#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::{
        ownable,
        ownable_two_step::{IOwnable2Step, Ownable2Step},
    },
    token::erc20::{self, Erc20, IErc20},
};
use stylus_sdk::prelude::*;

#[derive(SolidityError, Debug)]
enum Error {
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    InvalidSender(erc20::ERC20InvalidSender),
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    InvalidSpender(erc20::ERC20InvalidSpender),
    InvalidApprover(erc20::ERC20InvalidApprover),
    UnauthorizedAccount(ownable::OwnableUnauthorizedAccount),
    InvalidOwner(ownable::OwnableInvalidOwner),
}

impl From<erc20::Error> for Error {
    fn from(value: erc20::Error) -> Self {
        match value {
            erc20::Error::InsufficientBalance(e) => {
                Error::InsufficientBalance(e)
            }
            erc20::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc20::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc20::Error::InsufficientAllowance(e) => {
                Error::InsufficientAllowance(e)
            }
            erc20::Error::InvalidSpender(e) => Error::InvalidSpender(e),
            erc20::Error::InvalidApprover(e) => Error::InvalidApprover(e),
        }
    }
}

impl From<ownable::Error> for Error {
    fn from(value: ownable::Error) -> Self {
        match value {
            ownable::Error::UnauthorizedAccount(e) => {
                Error::UnauthorizedAccount(e)
            }
            ownable::Error::InvalidOwner(e) => Error::InvalidOwner(e),
        }
    }
}

#[entrypoint]
#[storage]
struct Ownable2StepExample {
    erc20: Erc20,
    ownable: Ownable2Step,
}

#[public]
#[implements(IErc20<Error = Error>, IOwnable2Step<Error = Error>)]
impl Ownable2StepExample {
    #[constructor]
    fn constructor(&mut self, initial_owner: Address) -> Result<(), Error> {
        Ok(self.ownable.constructor(initial_owner)?)
    }
}

#[public]
impl IErc20 for Ownable2StepExample {
    type Error = Error;

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        self.ownable.ownable.only_owner()?;
        Ok(self.erc20.transfer(to, value)?)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.approve(spender, value)?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Self::Error> {
        Ok(self.erc20.transfer_from(from, to, value)?)
    }
}

#[public]
impl IOwnable2Step for Ownable2StepExample {
    type Error = Error;

    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn pending_owner(&self) -> Address {
        self.ownable.pending_owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    fn accept_ownership(&mut self) -> Result<(), Self::Error> {
        Ok(self.ownable.accept_ownership()?)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        Ok(self.ownable.renounce_ownership()?)
    }
}
