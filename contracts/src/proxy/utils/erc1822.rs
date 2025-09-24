//! Universal Upgradeable Proxy Standard (UUPS) as defined in
//! [ERC-1822]: <https://eips.ethereum.org/EIPS/eip-1822>.

use alloc::vec::Vec;

use alloy_primitives::aliases::B256;
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::prelude::public;

/// Public interface for Universal Upgradeable Proxy Standard (UUPS).
///
/// This interface documents a method for upgradeability through a simplified
/// proxy whose upgrades are fully controlled by the current implementation.
#[interface_id]
#[public]
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

pub use erc1822_sol::Erc1822ProxiableInterface;

mod erc1822_sol {
    //! Solidity Interface of the ERC-1822 proxiable.

    #![allow(missing_docs)]
    #![cfg_attr(coverage_nightly, coverage(off))]

    use alloc::vec;

    use stylus_sdk::prelude::sol_interface;

    sol_interface! {
        interface Erc1822ProxiableInterface {
            function proxiableUUID() external view returns (bytes32);
        }
    }
}
