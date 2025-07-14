//! Solidity Interface of `BeaconProxy`.

use alloy_primitives::Address;

pub mod proxy;
pub mod upgradeable;

pub use beacon::IBeaconInterface;

/// This is the interface that [BeaconProxy][BeaconProxy] expects of its beacon.
///
/// [BeaconProxy]: crate::proxy::beacon::BeaconProxy
pub trait IBeacon {
    /// Must return an address that can be used as a delegate call target.
    ///
    /// [`UpgradeableBeacon`] will check that this address is a contract.
    fn implementation(&self) -> Result<Address, stylus_sdk::call::Error>;
}

mod beacon {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;
    sol_interface! {
        interface IBeaconInterface {
            function implementation() external view returns (address);
        }
    }
}
