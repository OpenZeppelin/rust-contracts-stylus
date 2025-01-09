#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{capped, Capped, Erc20Metadata, IErc20Burnable},
        Erc20, IErc20,
    },
    utils::{introspection::erc165::IErc165, Pausable},
};
use stylus_sdk::prelude::{entrypoint, public, storage};

const DECIMALS: u8 = 10;

#[entrypoint]
#[storage]
struct Erc20Example {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub capped: Capped,
    #[borrow]
    pub pausable: Pausable,
}

#[public]
#[inherit(Erc20, Erc20Metadata, Capped, Pausable)]
impl Erc20Example {
    // Overrides the default [`Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    pub fn decimals(&self) -> u8 {
        DECIMALS
    }

    pub fn burn(&mut self, value: U256) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.burn(value).map_err(|e| e.into())
    }

    pub fn burn_from(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.burn_from(account, value).map_err(|e| e.into())
    }

    // Add token minting feature.
    //
    // Make sure to handle `Capped` properly. You should not call
    // [`Erc20::_update`] to mint tokens -- it will the break `Capped`
    // mechanism.
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
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

    pub fn transfer(
        &mut self,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        self.pausable.when_not_paused()?;
        self.erc20.transfer(to, value).map_err(|e| e.into())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
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
    pub fn pause(&mut self) -> Result<(), Vec<u8>> {
        self.pausable.pause().map_err(|e| e.into())
    }

    pub fn unpause(&mut self) -> Result<(), Vec<u8>> {
        self.pausable.unpause().map_err(|e| e.into())
    }
}
