#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use openzeppelin_stylus::token::erc20::utils::safe_erc20::SafeErc20;
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
impl SafeErc20Example {}
