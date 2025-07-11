use alloc::{vec, vec::Vec};

use stylus_sdk::{prelude::*, storage::StorageAddress};

use crate::{access::ownable::IOwnable, proxy::beacon::IBeacon};

/// This contract is used in conjunction with one or more instances of
/// [BeaconProxy][BeaconProxy] to determine their implementation contract, which
/// is where they will delegate all function calls.
///
/// An owner is able to change the implementation the beacon points to, thus
/// upgrading the proxies that use this beacon.
///
/// [BeaconProxy]: crate::proxy::beacon::BeaconProxy
pub trait IUpgradeableBeacon: IBeacon + IOwnable {}

/// State of an [`UpgradeableBeacon`] contract.
#[storage]
pub struct UpgradeableBeacon {
    /// The address of the implementation contract.
    implementation: StorageAddress,
    /// The address of the owner of the contract.
    owner: StorageAddress,
}
