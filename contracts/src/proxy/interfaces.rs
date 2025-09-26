//! Consolidated Solidity Interfaces for proxy contracts.
//!
//! This module contains both callable and non-callable interfaces:
//! - **Callable interfaces**: defined with `stylus_proc::sol_interface`, which
//!   enables invoking contract functions directly
//! - **Non-callable interfaces**: defined with `alloy_sol_types::sol`, which
//!   enables constructing function call data to use with `RawCall`

pub use callable::*;

/// Callable interfaces defined with `stylus_proc::sol_interface`.
/// These enable invoking contract functions directly on the interface.
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
