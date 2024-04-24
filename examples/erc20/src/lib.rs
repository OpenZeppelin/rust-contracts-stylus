#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloc::string::String;

use alloy_primitives::{Address, U256};
use contracts::{
    erc20::{self, extensions::Metadata, ERC20InvalidReceiver, ERC20},
    erc20_capped_impl, erc20_impl, erc20_pausable_impl,
    utils::pausable::{IPausable, Pausable},
};
use erc20_proc::{
    ICapped, IERC20Burnable, IERC20Capped, IERC20Pausable, IERC20Storage,
    IERC20Virtual, IPausable, IERC20,
};
use stylus_sdk::prelude::{entrypoint, external, sol_storage};
const DECIMALS: u8 = 10;
use contracts::{
    erc20::{extensions::burnable::IERC20Burnable, IERC20Virtual, IERC20},
    erc20_burnable_impl,
    utils::capped::{Capped, ICapped},
};

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        BurnableCappedPausableERC20 erc20;
        #[borrow]
        Metadata metadata;
    }

    #[derive(IERC20Storage, IERC20, IERC20Virtual, IERC20Burnable, IPausable, ICapped)]
    struct BurnableCappedPausableERC20 {
        CappedPausableERC20 erc20;
    }

    #[derive(IERC20Storage, IERC20, IPausable, IERC20Capped)]
    struct CappedPausableERC20 {
        PausableERC20 erc20;
        Capped capped;
    }

    #[derive(IERC20Storage, IERC20, IERC20Pausable)]
    struct PausableERC20 {
        ERC20 erc20;
        Pausable pausable
    }
}

#[external]
#[inherit(Metadata)]
impl Token {
    // Export ERC-20 features
    erc20_impl!();

    // Export ERC-20 Burnable features
    erc20_burnable_impl!();

    // Export ERC-20 Capped features
    erc20_capped_impl!();

    // Export ERC-20 Pausable features
    erc20_pausable_impl!();

    // Add `mint` feature
    fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), contracts::erc20::Error> {
        if account.is_zero() {
            return Err(contracts::erc20::Error::InvalidReceiver(
                contracts::erc20::ERC20InvalidReceiver {
                    receiver: Address::ZERO,
                },
            ));
        }
        self.erc20._update(Address::ZERO, account, value)
    }

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
}
