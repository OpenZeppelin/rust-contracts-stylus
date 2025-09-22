#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    token::erc20::{
        self,
        extensions::{
            capped, Capped, Erc20Metadata, ICapped, IErc20Burnable,
            IErc20Metadata,
        },
        Erc20, IErc20,
    },
    utils::{introspection::erc165::IErc165, pausable, IPausable, Pausable},
};
use stylus_sdk::{
    alloy_primitives::{aliases::B32, uint, Address, U256, U8},
    prelude::*,
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
    metadata: Erc20Metadata,
    capped: Capped,
    pausable: Pausable,
}

#[public]
#[implements(IErc20<Error = Error>, IErc20Burnable<Error = Error>, IErc20Metadata, ICapped, IPausable, IErc165)]
impl Erc20Example {
    #[constructor]
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        cap: U256,
    ) -> Result<(), Error> {
        self.metadata.constructor(name, symbol);
        self.capped.constructor(cap)?;
        Ok(())
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

#[public]
impl IErc20 for Erc20Example {
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
        self.pausable.when_not_paused()?;
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
        self.pausable.when_not_paused()?;
        Ok(self.erc20.transfer_from(from, to, value)?)
    }
}

#[public]
impl IErc20Metadata for Erc20Example {
    fn name(&self) -> String {
        self.metadata.name()
    }

    fn symbol(&self) -> String {
        self.metadata.symbol()
    }

    // Overrides the default [`IErc20Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    fn decimals(&self) -> U8 {
        DECIMALS
    }
}

#[public]
impl IErc165 for Erc20Example {
    fn supports_interface(&self, interface_id: B32) -> bool {
        Erc20::supports_interface(&self.erc20, interface_id)
            || Erc20Metadata::supports_interface(&self.metadata, interface_id)
    }
}

#[public]
impl IErc20Burnable for Erc20Example {
    type Error = Error;

    fn burn(&mut self, value: U256) -> Result<(), Self::Error> {
        self.pausable.when_not_paused()?;
        Ok(self.erc20.burn(value)?)
    }

    fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Self::Error> {
        self.pausable.when_not_paused()?;
        Ok(self.erc20.burn_from(account, value)?)
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
