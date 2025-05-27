#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use openzeppelin_stylus::finance::vesting_wallet::{
    self, IVestingWallet, VestingWallet,
};
use stylus_sdk::{
    alloy_primitives::{Address, U256, U64},
    prelude::*,
};

#[entrypoint]
#[storage]
struct VestingWalletExample {
    vesting_wallet: VestingWallet,
}

#[public]
#[implements(IVestingWallet<Error = vesting_wallet::Error>)]
impl VestingWalletExample {
    #[constructor]
    pub fn constructor(
        &mut self,
        beneficiary: Address,
        start_timestamp: U64,
        duration_seconds: U64,
    ) -> Result<(), vesting_wallet::Error> {
        self.vesting_wallet.constructor(
            beneficiary,
            start_timestamp,
            duration_seconds,
        )
    }

    #[receive]
    fn receive(&mut self) -> Result<(), Vec<u8>> {
        self.vesting_wallet.receive()
    }
}

#[public]
impl IVestingWallet for VestingWalletExample {
    type Error = vesting_wallet::Error;

    fn owner(&self) -> Address {
        self.vesting_wallet.owner()
    }

    fn transfer_ownership(
        &mut self,
        new_owner: Address,
    ) -> Result<(), Self::Error> {
        self.vesting_wallet.transfer_ownership(new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Self::Error> {
        self.vesting_wallet.renounce_ownership()
    }

    fn start(&self) -> U256 {
        self.vesting_wallet.start()
    }

    fn duration(&self) -> U256 {
        self.vesting_wallet.duration()
    }

    fn end(&self) -> U256 {
        self.vesting_wallet.end()
    }

    #[selector(name = "released")]
    fn released_eth(&self) -> U256 {
        self.vesting_wallet.released_eth()
    }

    #[selector(name = "released")]
    fn released_erc20(&self, token: Address) -> U256 {
        self.vesting_wallet.released_erc20(token)
    }

    #[selector(name = "releasable")]
    fn releasable_eth(&self) -> U256 {
        self.vesting_wallet.releasable_eth()
    }

    #[selector(name = "releasable")]
    fn releasable_erc20(
        &mut self,
        token: Address,
    ) -> Result<U256, Self::Error> {
        self.vesting_wallet.releasable_erc20(token)
    }

    #[selector(name = "release")]
    fn release_eth(&mut self) -> Result<(), Self::Error> {
        self.vesting_wallet.release_eth()
    }

    #[selector(name = "release")]
    fn release_erc20(&mut self, token: Address) -> Result<(), Self::Error> {
        self.vesting_wallet.release_erc20(token)
    }

    #[selector(name = "vestedAmount")]
    fn vested_amount_eth(&self, timestamp: u64) -> U256 {
        self.vesting_wallet.vested_amount_eth(timestamp)
    }

    #[selector(name = "vestedAmount")]
    fn vested_amount_erc20(
        &mut self,
        token: Address,
        timestamp: u64,
    ) -> Result<U256, Self::Error> {
        self.vesting_wallet.vested_amount_erc20(token, timestamp)
    }
}
