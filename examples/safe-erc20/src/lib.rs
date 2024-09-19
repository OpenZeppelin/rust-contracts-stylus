#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::token::erc20::utils::safe_erc20::{Error, SafeErc20};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct SafeErc20Example {
        #[borrow]
        SafeErc20 safe_erc20;
    }
}

#[public]
#[inherit(SafeErc20)]
impl SafeErc20Example {
    // Add token minting feature.
    pub fn safe_transfer_token(
        &mut self,
        token: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error> {
        self.safe_erc20.safe_transfer(token, to, value)?;
        Ok(())
    }
}
