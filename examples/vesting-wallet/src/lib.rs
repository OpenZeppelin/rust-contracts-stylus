#![cfg_attr(not(test), no_main)]
extern crate alloc;

use openzeppelin_stylus::finance::vesting_wallet::VestingWallet;
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct VestingWalletExample {
    #[borrow]
    pub vesting_wallet: VestingWallet,
}

#[public]
#[inherit(VestingWallet)]
impl VestingWalletExample {}
