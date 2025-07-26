#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc20::{
        self,
        extensions::{
            capped, Capped, Erc20MetadataStorage, ICapped, IErc20Burnable,
            IErc20Metadata,
        },
        Erc20, Erc20Internal, Erc20Storage, IErc20,
    },
    utils::{introspection::erc165::IErc165, pausable, IPausable, Pausable},
};
use stylus_sdk::{
    alloy_primitives::{uint, Address, FixedBytes, U256, U8},
    prelude::*,
    storage::{StorageMap, StorageU256},
};

const DECIMALS: U8 = uint!(10_U8);

#[derive(SolidityError, Debug)]
enum Error {
    ExceededCap(capped::ERC20ExceededCap),
    InvalidCap(capped::ERC20InvalidCap),
    InsufficientBalance(erc20::ERC20InsufficientBalance),
    InvalidSender(erc20::ERC20InvalidSender),
    InvalidReceiver(erc20::ERC20InvalidReceiver),
    InsufficientAllowance(erc20::ERC20InsufficientAllowance),
    InvalidSpender(erc20::ERC20InvalidSpender),
    InvalidApprover(erc20::ERC20InvalidApprover),
    EnforcedPause(pausable::EnforcedPause),
    ExpectedPause(pausable::ExpectedPause),
}

impl From<capped::Error> for Error {
    fn from(value: capped::Error) -> Self {
        match value {
            capped::Error::ExceededCap(e) => Error::ExceededCap(e),
            capped::Error::InvalidCap(e) => Error::InvalidCap(e),
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

impl From<pausable::Error> for Error {
    fn from(value: pausable::Error) -> Self {
        match value {
            pausable::Error::EnforcedPause(e) => Error::EnforcedPause(e),
            pausable::Error::ExpectedPause(e) => Error::ExpectedPause(e),
        }
    }
}

#[entrypoint]
#[storage]
struct Erc20Example {
    erc20: Erc20,
    capped: Capped,
    pausable: Pausable,
}

#[public]
#[implements(
    IErc20,
    IErc20Burnable,
    IErc20Metadata,
    ICapped,
    IPausable,
    IErc165
)]
impl Erc20Example {
    #[constructor]
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        cap: U256,
    ) -> Result<(), Error> {
        self.erc20.constructor(name, symbol);
        self.capped.constructor(cap)?;
        Ok(())
    }

    // Add token minting feature.
    //
    // Make sure to handle `Capped` properly. You should not call
    // [`Erc20::_update`] to mint tokens -- it will the break `Capped`
    // mechanism.
    fn mint(&mut self, account: Address, value: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        let max_supply = self.capped.cap();

        // Overflow check required.
        let supply = IErc20::total_supply(self)
            .checked_add(value)
            .expect("new supply should not exceed `U256::MAX`");

        if supply > max_supply {
            return Err(capped::Error::ExceededCap(
                capped::ERC20ExceededCap {
                    increased_supply: supply,
                    cap: max_supply,
                },
            ))?;
        }

        self.erc20._mint(account, value)?;
        Ok(())
    }

    /// WARNING: These functions are intended for **testing purposes** only. In
    /// **production**, ensure strict access control to prevent unauthorized
    /// pausing or unpausing, which can disrupt contract functionality. Remove
    /// or secure these functions before deployment.
    fn pause(&mut self) -> Result<(), Error> {
        Ok(self.pausable.pause()?)
    }

    fn unpause(&mut self) -> Result<(), Error> {
        Ok(self.pausable.unpause()?)
    }
}

impl Erc20Storage for Erc20Example {
    fn balances(&self) -> &StorageMap<Address, StorageU256> {
        &self.erc20.balances
    }

    fn balances_mut(&mut self) -> &mut StorageMap<Address, StorageU256> {
        &mut self.erc20.balances
    }

    fn allowances(
        &self,
    ) -> &StorageMap<Address, StorageMap<Address, StorageU256>> {
        &self.erc20.allowances
    }

    fn allowances_mut(
        &mut self,
    ) -> &mut StorageMap<Address, StorageMap<Address, StorageU256>> {
        &mut self.erc20.allowances
    }

    fn total_supply(&self) -> &StorageU256 {
        &self.erc20.total_supply
    }

    fn total_supply_mut(&mut self) -> &mut StorageU256 {
        &mut self.erc20.total_supply
    }
}

impl Erc20Internal for Erc20Example {}

#[public]
impl IErc20 for Erc20Example {
    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.transfer(to, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.transfer_from(from, to, value)
    }
}

#[public]
impl IErc20Metadata for Erc20Example {
    // Overrides the default [`IErc20Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    fn decimals(&self) -> U8 {
        DECIMALS
    }
}

impl Erc20MetadataStorage for Erc20Example {
    fn name(&self) -> &stylus_sdk::storage::StorageString {
        &self.erc20.metadata.name
    }

    fn symbol(&self) -> &stylus_sdk::storage::StorageString {
        &self.erc20.metadata.symbol
    }
}

#[public]
impl IErc165 for Erc20Example {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        Erc20::supports_interface(&self.erc20, interface_id)
    }
}

#[public]
impl IErc20Burnable for Erc20Example {
    fn burn(&mut self, value: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.burn(value)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.burn_from(account, value)
    }
}

#[public]
impl ICapped for Erc20Example {
    fn cap(&self) -> U256 {
        self.capped.cap()
    }
}

#[public]
impl IPausable for Erc20Example {
    fn paused(&self) -> bool {
        self.pausable.paused()
    }
}
