#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, U64};
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
impl VestingWalletExample {
    #[constructor]
    pub fn constructor(
        &mut self,
        _beneficiary: Address,
        _start_timestamp: U64,
        _duration_seconds: U64,
    ) -> Result<(), Vec<u8>> {
        todo!()
        // Ok(self.vesting_wallet.constructor(initial_owner)?)
    }
}
