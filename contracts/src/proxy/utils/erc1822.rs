//! Universal Upgradeable Proxy Standard (UUPS) as defined in
//! [ERC-1822]: <https://eips.ethereum.org/EIPS/eip-1822>.

use alloc::vec::Vec;

use alloy_primitives::aliases::B256;
use openzeppelin_stylus_proc::interface_id;

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
    /// * May return an error based on the implementation.
    #[selector(name = "proxiableUUID")]
    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>>;
}

pub use crate::proxy::abi::Erc1822ProxiableInterface;
