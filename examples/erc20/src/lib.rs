#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, U256};
use contracts::{
    erc20::{extensions::ERC20Metadata, ERC20},
    erc20_burnable_impl,
    utils::Capped,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

const DECIMALS: u8 = 10;

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        ERC20 erc20;
        #[borrow]
        ERC20Metadata metadata;
        #[borrow]
        Capped capped;
    }
}

#[external]
#[inherit(ERC20, ERC20Metadata, Capped)]
impl Token {
    // This macro implements ERC20Burnable functions -- `burn` and `burn_from`.
    // Expects an `ERC20 erc20` as a field of `Token`.
    erc20_burnable_impl!();

    // We need to properly initialize all Token's attributes.
    // For that we need to call each attributes' constructor if exists.
    //
    // NOTE: This is a temporary solution for state initialization.
    pub fn constructor(
        &mut self,
        name: String,
        symbol: String,
        cap: U256,
    ) -> Result<(), Vec<u8>> {
        self.metadata.constructor(name, symbol);
        self.capped.constructor(cap)?;
        Ok(())
    }

    // Overrides the default [`Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    pub fn decimals(&self) -> u8 {
        DECIMALS
    }

    // Add token minting feature.
    // Make sure to handle `Capped` properly.
    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        let max_supply = self.capped.cap();
        let supply = self.erc20.total_supply() + value;
        if supply > max_supply {
            return Err(contracts::utils::capped::Error::ExceededCap(
                contracts::utils::capped::ExceededCap {
                    increased_supply: supply,
                    cap: max_supply,
                },
            ))?;
        }

        self.erc20._mint(account, value)?;
        Ok(())
    }
}
