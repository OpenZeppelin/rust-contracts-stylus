#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U256};
use openzeppelin_stylus::{
    token::erc20::{
        extensions::{capped, Capped, Erc20Metadata, IErc20Burnable, Permit},
        Erc20, IErc20, IErc20Internal,
    },
    utils::{cryptography::eip712::IEip712, Pausable},
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

const DECIMALS: u8 = 10;

sol_storage! {
    #[entrypoint]
    struct Erc20Example {
        #[borrow]
        Erc20 erc20;
        #[borrow]
        Erc20Metadata metadata;
        #[borrow]
        Capped capped;
        #[borrow]
        Pausable pausable;
        #[borrow]
        Permit<Eip712> permit;
    }

    struct Eip712 {}
}

impl IEip712 for Eip712 {
    const NAME: &'static str = "ERC-20 Permit Example";
    const VERSION: &'static str = "1";
}

#[external]
#[inherit(Erc20, Erc20Metadata, Capped, Pausable, Permit<Eip712>)]
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

    #[allow(clippy::too_many_arguments)]
    pub fn permit(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        deadline: U256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<(), Vec<u8>> {
        self.permit
            .permit(owner, spender, value, deadline, v, r, s, &mut self.erc20)
            .map_err(|e| e.into())
    }
}
