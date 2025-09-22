#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    access::control::{
        self,
        extensions::{AccessControlEnumerable, IAccessControlEnumerable},
        AccessControl, IAccessControl,
    },
    token::erc20::{self, Erc20, IErc20},
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    alloy_primitives::{aliases::B32, Address, B256, U256},
    msg,
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
struct AccessControlExample {
    erc20: Erc20,
    access: AccessControl,
    access_enumerable: AccessControlEnumerable,
}

pub const TRANSFER_ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"TRANSFER_ROLE").finalize();

#[public]
#[implements(IErc20<Error = Error>, IAccessControl<Error = control::Error>, IAccessControlEnumerable<Error = control::extensions::enumerable::Error>, IErc165)]
impl AccessControlExample {
    #[constructor]
    fn constructor(&mut self, admin: Address) {
        self.access_enumerable._grant_role(
            AccessControl::DEFAULT_ADMIN_ROLE.into(),
            admin,
            &mut self.access,
        );
    }

    fn make_admin(&mut self, account: Address) -> Result<(), Error> {
        self.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())?;
        self.grant_role(TRANSFER_ROLE.into(), account)?;
        Ok(())
    }

    // WARNING: This should not be part of the public API, it's here for testing
    // purposes only.
    fn set_role_admin(&mut self, role: B256, new_admin_role: B256) {
        self.access._set_role_admin(role, new_admin_role);
    }

    fn get_role_members(&self, role: B256) -> Vec<Address> {
        self.access_enumerable.get_role_members(role)
    }
}

#[public]
impl IErc20 for AccessControlExample {
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
impl IAccessControl for AccessControlExample {
    type Error = control::Error;

    fn has_role(&self, role: B256, account: Address) -> bool {
        self.access.has_role(role, account)
    }

    fn only_role(&self, role: B256) -> Result<(), Self::Error> {
        self.access.only_role(role)
    }

    fn get_role_admin(&self, role: B256) -> B256 {
        self.access.get_role_admin(role)
    }

    fn grant_role(
        &mut self,
        role: B256,
        account: Address,
    ) -> Result<(), Self::Error> {
        let admin_role = self.get_role_admin(role);
        self.only_role(admin_role)?;
        self.access_enumerable._grant_role(role, account, &mut self.access);
        Ok(())
    }

    fn revoke_role(
        &mut self,
        role: B256,
        account: Address,
    ) -> Result<(), Self::Error> {
        let admin_role = self.get_role_admin(role);
        self.only_role(admin_role)?;
        self.access_enumerable._revoke_role(role, account, &mut self.access);
        Ok(())
    }

    #[allow(deprecated)]
    fn renounce_role(
        &mut self,
        role: B256,
        confirmation: Address,
    ) -> Result<(), Self::Error> {
        if msg::sender() != confirmation {
            return Err(control::Error::BadConfirmation(
                control::AccessControlBadConfirmation {},
            ));
        }

        self.access_enumerable._revoke_role(
            role,
            confirmation,
            &mut self.access,
        );
        Ok(())
    }
}

#[public]
impl IAccessControlEnumerable for AccessControlExample {
    type Error = control::extensions::enumerable::Error;

    fn get_role_member(
        &self,
        role: B256,
        index: U256,
    ) -> Result<Address, Self::Error> {
        self.access_enumerable.get_role_member(role, index)
    }

    fn get_role_member_count(&self, role: B256) -> U256 {
        self.access_enumerable.get_role_member_count(role)
    }
}

#[public]
impl IErc165 for AccessControlExample {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.access.supports_interface(interface_id)
            || self.access_enumerable.supports_interface(interface_id)
            || self.erc20.supports_interface(interface_id)
    }
}
