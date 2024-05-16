#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use alloy_primitives::{Address, U256};
use contracts::{
    erc20::{extensions::ERC20Metadata, ERC20, Error, ERC20InvalidReceiver},
    erc20_burnable_impl,
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
    }
}

#[external]
#[inherit(ERC20, ERC20Metadata)]
impl Token {
    // This macro implements ERC20Burnable functions -- `burn` and `burn_from`.
    // Expects an `ERC20 erc20` as a field of `Token`.
    erc20_burnable_impl!();

    pub fn constructor(&mut self, name: String, symbol: String) {
        self.metadata.constructor(name, symbol);
    }

    // Overrides the default [`Metadata::decimals`], and sets it to `10`.
    //
    // If you don't provide this method in the `entrypoint` contract, it will
    // default to `18`.
    pub fn decimals(&self) -> u8 {
        DECIMALS
    }

    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Error> {
        // TODO: create function _mint at erc20 similar to solidity
        if account.is_zero() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }
        self.erc20._update(Address::ZERO, account, value)
    }
}
