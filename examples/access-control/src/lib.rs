#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    access::control::{self, AccessControl, IAccessControl},
    token::erc20::{self, Erc20, IErc20},
};
use stylus_sdk::{
    alloy_primitives::{Address, B256, U256},
    prelude::*,
};

#[derive(SolidityError, Debug)]
enum Error {
    UnauthorizedAccount(control::AccessControlUnauthorizedAccount),
    BadConfirmation(control::AccessControlBadConfirmation),
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    InvalidSender(erc20::ERC20InvalidSender),
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    InvalidSpender(erc20::ERC20InvalidSpender),
    InvalidApprover(erc20::ERC20InvalidApprover),
}

impl From<control::Error> for Error {
    fn from(value: control::Error) -> Self {
        match value {
            control::Error::UnauthorizedAccount(e) => {
                Error::UnauthorizedAccount(e)
            }
            control::Error::BadConfirmation(e) => Error::BadConfirmation(e),
        }
    }
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

#[entrypoint]
#[storage]
struct Example {
    erc20: Erc20,
    access: AccessControl,
}

const MINTER_ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"MINTER_ROLE").finalize();

#[public]
#[implements(IErc20<Error = Error>, IAccessControl<Error = Error>)]
impl Example {
    fn mint(&mut self, to: Address, amount: U256) -> Result<(), Error> {
        self.access.only_role(MINTER_ROLE.into())?;
        self.erc20._mint(to, amount)?;
        Ok(())
    }
}

#[public]
impl IErc20 for Example {
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
        self.access.only_role(TRANSFER_ROLE.into())?;
        let transfer_result = self.erc20.transfer_from(from, to, value)?;
        Ok(transfer_result)
    }
}

#[public]
impl IAccessControl for Example {
    type Error = Error;

    fn has_role(&self, role: B256, account: Address) -> bool {
        self.access.has_role(role, account)
    }

    fn only_role(&self, role: B256) -> Result<(), Self::Error> {
        Ok(self.access.only_role(role)?)
    }

    fn get_role_admin(&self, role: B256) -> B256 {
        self.access.get_role_admin(role)
    }

    fn grant_role(
        &mut self,
        role: B256,
        account: Address,
    ) -> Result<(), Self::Error> {
        Ok(self.access.grant_role(role, account)?)
    }

    fn revoke_role(
        &mut self,
        role: B256,
        account: Address,
    ) -> Result<(), Self::Error> {
        Ok(self.access.revoke_role(role, account)?)
    }

    fn renounce_role(
        &mut self,
        role: B256,
        confirmation: Address,
    ) -> Result<(), Self::Error> {
        Ok(self.access.renounce_role(role, confirmation)?)
    }
}
