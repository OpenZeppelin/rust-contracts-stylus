#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use openzeppelin_stylus::finance::vesting_wallet::VestingWallet;
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct VestingWalletExample {
        #[borrow]
        VestingWallet vesting_wallet;
    }
}

#[public]
#[inherit(VestingWallet)]
impl VestingWalletExample {}
