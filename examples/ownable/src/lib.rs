#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::ownable::{self, Ownable},
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
struct OwnableExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    ownable: Ownable,
}

#[public]
#[inherit(Erc20, Ownable)]
impl OwnableExample {
    fn transfer(&mut self, to: Address, value: U256) -> Result<(), Error> {
        self.ownable.only_owner()?;
        self.erc20.transfer(to, value)?;
        Ok(())
    }
}
