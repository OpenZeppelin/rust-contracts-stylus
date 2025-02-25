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
        beneficiary: Address,
        start_timestamp: U64,
        duration_seconds: U64,
    ) -> Result<(), Vec<u8>> {
        Ok(self.vesting_wallet.constructor(
            beneficiary,
            start_timestamp,
            duration_seconds,
        )?)
    }

    #[receive]
    fn receive(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }
}
