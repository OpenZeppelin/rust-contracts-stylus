#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    access::{
        control::{self, AccessControl, IAccessControl},
        enumerable::{self, AccessControlEnumerable, IAccessControlEnumerable},
    },
    token::erc20::{self, Erc20, IErc20},
};
use stylus_sdk::prelude::*;

/// Example error type combining errors from all used components
#[derive(SolidityError, Debug)]
enum Error {
    AccessControl(control::Error),
    AccessControlEnumerable(enumerable::Error),
    Erc20(erc20::Error),
}

/// Example contract demonstrating AccessControlEnumerable usage
#[entrypoint]
#[storage]
struct AccessControlEnumerableExample {
    #[borrow]
    erc20: Erc20,
    #[borrow]
    access: AccessControlEnumerable,
}

/// Role definitions using keccak256 hashes
pub const MINTER_ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"MINTER_ROLE").finalize();
pub const BURNER_ROLE: [u8; 32] =
    keccak_const::Keccak256::new().update(b"BURNER_ROLE").finalize();

#[public]
#[inherit(Erc20, AccessControl, AccessControlEnumerable)]
impl AccessControlEnumerableExample {
    /// Initializes the token contract with a name and symbol.
    /// Grants DEFAULT_ADMIN_ROLE to the deployer.
    #[payable(false)]
    #[fallback]
    fn constructor(&mut self, name: String, symbol: String) -> Result<(), Error> {
        // Initialize ERC20 token
        self.erc20.init(name, symbol, 18)?;

        // Setup the DEFAULT_ADMIN_ROLE to the deployer
        let sender = msg::sender();
        self.access.grant_role(AccessControl::DEFAULT_ADMIN_ROLE.into(), sender)
            .map_err(Error::AccessControlEnumerable)?;

        Ok(())
    }

    /// Adds a minter role to the specified account.
    /// Only callable by an account with the DEFAULT_ADMIN_ROLE.
    fn add_minter(&mut self, account: Address) -> Result<(), Error> {
        // Only admin can add minters
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())
            .map_err(Error::AccessControl)?;

        self.access.grant_role(MINTER_ROLE.into(), account)
            .map_err(Error::AccessControlEnumerable)
    }

    /// Adds a burner role to the specified account.
    /// Only callable by an account with the DEFAULT_ADMIN_ROLE.
    fn add_burner(&mut self, account: Address) -> Result<(), Error> {
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())
            .map_err(Error::AccessControl)?;

        self.access.grant_role(BURNER_ROLE.into(), account)
            .map_err(Error::AccessControlEnumerable)
    }

    /// Removes the minter role from the specified account.
    /// Only callable by an account with the DEFAULT_ADMIN_ROLE.
    fn remove_minter(&mut self, account: Address) -> Result<(), Error> {
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())
            .map_err(Error::AccessControl)?;

        self.access.revoke_role(MINTER_ROLE.into(), account)
            .map_err(Error::AccessControlEnumerable)
    }

    /// Removes the burner role from the specified account.
    /// Only callable by an account with the DEFAULT_ADMIN_ROLE.
    fn remove_burner(&mut self, account: Address) -> Result<(), Error> {
        self.access.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())
            .map_err(Error::AccessControl)?;

        self.access.revoke_role(BURNER_ROLE.into(), account)
            .map_err(Error::AccessControlEnumerable)
    }

    /// Returns a list of all accounts that have been granted the minter role.
    /// Demonstrates how to enumerate role members using AccessControlEnumerable.
    fn get_minters(&self) -> Vec<Address> {
        let count = self.access.get_role_member_count(MINTER_ROLE.into());
        let mut minters = Vec::new();
        for i in 0..count.as_u64() {
            if let Ok(minter) = self.access.get_role_member(MINTER_ROLE.into(), i.into()) {
                minters.push(minter);
            }
        }
        minters
    }

    /// Returns a list of all accounts that have been granted the burner role.
    /// Demonstrates how to enumerate role members using AccessControlEnumerable.
    fn get_burners(&self) -> Vec<Address> {
        let count = self.access.get_role_member_count(BURNER_ROLE.into());
        let mut burners = Vec::new();
        for i in 0..count.as_u64() {
            if let Ok(burner) = self.access.get_role_member(BURNER_ROLE.into(), i.into()) {
                burners.push(burner);
            }
        }
        burners
    }

    /// Mints tokens to the specified address.
    /// Only callable by accounts with the MINTER_ROLE.
    fn mint(&mut self, to: Address, amount: U256) -> Result<(), Error> {
        self.access.only_role(MINTER_ROLE.into())
            .map_err(Error::AccessControl)?;

        self.erc20.mint(to, amount)
            .map_err(Error::Erc20)
    }

    /// Burns tokens from the specified address.
    /// Only callable by accounts with the BURNER_ROLE.
    fn burn(&mut self, from: Address, amount: U256) -> Result<(), Error> {
        self.access.only_role(BURNER_ROLE.into())
            .map_err(Error::AccessControl)?;

        self.erc20.burn(from, amount)
            .map_err(Error::Erc20)
    }
}
