//! Universal Upgradeable Proxy Standard (UUPS) as defined in
//! [ERC-1822]: <https://eips.ethereum.org/EIPS/eip-1822>.

use alloy_primitives::U256;
use openzeppelin_stylus_proc::interface_id;

pub mod uups_upgradeable;

use alloc::vec::Vec;

/// Public interface for Universal Upgradeable Proxy Standard (UUPS).
///
/// This interface documents a method for upgradeability through a simplified
/// proxy whose upgrades are fully controlled by the current implementation.
#[interface_id]
pub trait IErc1822Proxiable {
    /// Returns the storage slot that the proxiable contract assumes is being
    /// used to store the implementation address.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    ///
    /// # Errors
    ///
    /// TODO!
    #[selector(name = "proxiableUUID")]
    fn proxiable_uuid(&self) -> Result<U256, Vec<u8>>;
}
