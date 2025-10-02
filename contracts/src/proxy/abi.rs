//! Consolidated Solidity Interfaces for proxy contracts.
//!
//! This module contains both contract interfaces and ABI interfaces:
//! - **Contract interfaces**: defined with `stylus_proc::sol_interface`, which
//!   enables invoking contract functions directly on actual deployed contracts
//! - **ABI interfaces**: defined with `alloy_sol_types::sol`, which enables
//!   constructing function call data to use with `RawCall`

pub use callable::*;

/// Contract interfaces defined with `stylus_proc::sol_interface`.
/// These enable invoking contract functions directly on actual deployed
/// contracts.
mod callable {
    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        /// Beacon proxy interface.
        interface IBeaconInterface {
            function implementation() external view returns (address);
        }
    }

    sol_interface! {
        /// ERC-1822 Proxiable interface.
        interface Erc1822ProxiableInterface {
            function proxiableUUID() external view returns (bytes32);
        }
    }
}
