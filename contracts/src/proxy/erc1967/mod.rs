//! Proxy Storage Slots and the events as defined in
//! the [ERC-1967].

//! [ERC-1967]: <https://eips.ethereum.org/EIPS/eip-1967>
use alloc::{vec, vec::Vec};

use stylus_sdk::prelude::*;

pub mod proxy;
pub mod utils;

pub use sol::*;

use crate::proxy::IProxy;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the implementation is upgraded.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Upgraded(address indexed implementation);

        /// Emitted when the admin account has changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event AdminChanged(address indexed previous_admin, address indexed new_admin);

        /// Emitted when the beacon is changed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event BeaconUpgraded(address indexed beacon);
    }
}

#[storage]
pub struct Erc1967Proxy;

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc1967Proxy {}

/// Interface for the [`IErc1967Proxy`] contract.
pub trait IErc1967Proxy: IProxy + TopLevelStorage {}
