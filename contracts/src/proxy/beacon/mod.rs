//! Solidity Interface of `BeaconProxy`.
use alloc::vec::Vec;

use alloy_primitives::Address;

pub mod proxy;
pub mod upgradeable;

pub use beacon::IBeaconInterface;
use openzeppelin_stylus_proc::interface_id;

/// This is the interface that [BeaconProxy][BeaconProxy] expects of its beacon.
///
/// [BeaconProxy]: crate::proxy::beacon::BeaconProxy
#[interface_id]
pub trait IBeacon {
    /// Must return an address that can be used as a delegate call target.
    ///
    /// [`UpgradeableBeacon`] will check that this address is a contract.
    fn implementation(&self) -> Result<Address, Vec<u8>>;
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
