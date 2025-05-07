#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc20::{
        self,
        extensions::{capped, Capped, Erc20Metadata, IErc20Burnable},
        Erc20, IErc20,
    },
    utils::{introspection::erc165::IErc165, pausable, Pausable},
};
use stylus_sdk::prelude::*;

const DECIMALS: u8 = 10;

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
    #[borrow]
    erc20: Erc20,
    #[borrow]
    metadata: Erc20Metadata,
    #[borrow]
    capped: Capped,
    #[borrow]
    pausable: Pausable,
}

#[public]
#[inherit(Erc20, Erc20Metadata, Capped, Pausable)]
impl Erc20Example {
    // Overrides the default [`Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    fn decimals(&self) -> u8 {
        DECIMALS
    }

    fn burn(&mut self, value: U256) -> Result<(), Error> {
        self.pausable.when_not_paused()?;
        self.erc20.burn(value).map_err(|e| e.into())
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Error> {
        self.pausable.when_not_paused()?;
        self.erc20.burn_from(account, value).map_err(|e| e.into())
    }

    // Add token minting feature.
    //
    // Make sure to handle `Capped` properly. You should not call
    // [`Erc20::_update`] to mint tokens -- it will the break `Capped`
    // mechanism.
    fn mint(&mut self, account: Address, value: U256) -> Result<(), Error> {
        self.pausable.when_not_paused()?;
        let max_supply = self.capped.cap();

        // Overflow check required.
        let supply = self
            .erc20
            .total_supply()
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

    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Error> {
        self.pausable.when_not_paused()?;
        self.erc20.transfer(to, value).map_err(|e| e.into())
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Error> {
        self.pausable.when_not_paused()?;
        self.erc20.transfer_from(from, to, value).map_err(|e| e.into())
    }

    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        Erc20::supports_interface(interface_id)
            || Erc20Metadata::supports_interface(interface_id)
    }

    /// WARNING: These functions are intended for **testing purposes** only. In
    /// **production**, ensure strict access control to prevent unauthorized
    /// pausing or unpausing, which can disrupt contract functionality. Remove
    /// or secure these functions before deployment.
    fn pause(&mut self) -> Result<(), Error> {
        self.pausable.pause().map_err(|e| e.into())
    }

    fn unpause(&mut self) -> Result<(), Error> {
        self.pausable.unpause().map_err(|e| e.into())
    }
}
