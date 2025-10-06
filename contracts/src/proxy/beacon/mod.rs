//! Solidity Interface of [`BeaconProxy`].
use alloc::vec::Vec;

use alloy_primitives::Address;

pub mod proxy;
pub mod upgradeable;

use openzeppelin_stylus_proc::interface_id;
pub use proxy::BeaconProxy;
pub use upgradeable::{Error, IUpgradeableBeacon, UpgradeableBeacon};

pub use crate::proxy::abi::BeaconInterface;

/// This is the interface that [`BeaconProxy`] expects of its beacon.
#[interface_id]
pub trait IBeacon {
    /// Must return an address that can be used as a delegate call target.
    ///
    /// [`UpgradeableBeacon`] will check that this address is a contract.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// Implementing contracts should define their own error types for this
    /// function. Typically, errors may include:
    /// * The implementation address is invalid (e.g., not a contract).
    /// * The implementation is not a contract.
    ///
    /// The error should be encoded as a [`Vec<u8>`].
    fn implementation(&self) -> Result<Address, Vec<u8>>;
}
