#![cfg_attr(not(test), no_main)]
extern crate alloc;

use openzeppelin_stylus::token::erc20::utils::safe_erc20::SafeErc20;
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct SafeErc20Example {
    #[borrow]
    pub safe_erc20: SafeErc20,
}

#[public]
#[inherit(SafeErc20)]
impl SafeErc20Example {}
