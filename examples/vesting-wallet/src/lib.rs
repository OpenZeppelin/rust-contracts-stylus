#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus::{
    access::ownable::Ownable,
    finance::vesting_wallet::{Error, VestingWallet},
};
use stylus_sdk::prelude::{entrypoint, public, sol_storage};

sol_storage! {
    #[entrypoint]
    struct VestingWalletExample {
        #[borrow]
        Ownable ownable;
        #[borrow]
        VestingWallet vesting_wallet;
    }
}

#[public]
#[inherit(VestingWallet, Ownable)]
impl VestingWalletExample {
    /// The contract should be able to receive Eth.
    #[payable]
    pub fn receive_ether(&self) {}

    /// Getter for the start timestamp.
    pub fn start(&self) -> U256 {
        self.vesting_wallet.start()
    }

    /// Getter for the vesting duration.
    pub fn duration(&self) -> U256 {
        self.vesting_wallet.duration()
    }

    /// Getter for the end timestamp.
    pub fn end(&self) -> U256 {
        self.vesting_wallet.end()
    }

    #[selector(name = "released")]
    pub fn released_eth(&self) -> U256 {
        self.vesting_wallet.released_eth()
    }

    #[selector(name = "released")]
    pub fn released_erc20(&self, token: Address) -> U256 {
        self.vesting_wallet.released_erc20(token)
    }

    #[selector(name = "releasable")]
    pub fn releasable_eth(&self) -> U256 {
        self.vesting_wallet.releasable_eth()
    }

    #[selector(name = "releasable")]
    pub fn releasable_erc20(&mut self, token: Address) -> U256 {
        self.vesting_wallet.releasable_erc20(token)
    }

    #[selector(name = "vestedAmount")]
    pub fn vested_amount_eth(&self, timestamp: u64) -> U256 {
        self.vesting_wallet.vested_amount_eth(timestamp)
    }

    #[selector(name = "vestedAmount")]
    pub fn vested_amount_erc20(
        &mut self,
        token: Address,
        timestamp: u64,
    ) -> U256 {
        self.vesting_wallet.vested_amount_erc20(token, timestamp)
    }

    #[selector(name = "release")]
    pub fn release_eth(&mut self) -> Result<(), Error> {
        let owner = self.ownable.owner();
        self.vesting_wallet._release_eth(owner)
    }

    #[selector(name = "release")]
    pub fn release_erc20(&mut self, token: Address) -> Result<(), Error> {
        let owner = self.ownable.owner();
        self.vesting_wallet._release_erc20(owner, token)
    }
}
