//! Solidity Interface of `BeaconProxy`.

pub mod proxy;

pub use beacon::*;

mod beacon {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;
    sol_interface! {
        interface IBeacon {
            function implementation() external view returns (address);
        }
    }
}
