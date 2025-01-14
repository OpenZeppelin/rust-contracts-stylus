//! This library provides getters and event emitting update functions for
//! [ERC-1967] slots.
//!
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967

use alloy_primitives::{b256, Address, B256};
pub use sol::*;
use stylus_sdk::abi::Bytes;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Indicates an error related to the fact that the `implementation`
        /// of the proxy is invalid.
        ///
        /// * `implementation` - Address of the invalid implementation.
        error ERC1967InvalidImplementation(address implementation);

        /// Indicates an error related to the fact that the `admin` of the
        /// proxy is invalid.
        ///
        /// * `admin` - Address of the invalid admin.
        error ERC1967InvalidAdmin(address admin);

        /// Indicates an error related to the fact that the `beacon`
        /// of the proxy is invalid.
        ///
        /// * `beacon` - Address of the invalid `beacon` of the proxy.
        error ERC1967InvalidBeacon(address beacon);

        /// Indicates an error relatoed to the fact that an upgrade function
        /// sees `stylus_sdk::msg::value() > 0` that may be lost.
        error ERC1967NonPayable();
    }
}

/// Storage slot with the address of the current implementation.
/// This is the keccak-256 hash of "eip1967.proxy.implementation" subtracted by
/// 1.
const IMPLEMENTATION_SLOT: B256 =
    b256!("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");

/// Storage slot with the admin of the contract.
/// This is the keccak-256 hash of "eip1967.proxy.admin" subtracted by 1.
const ADMIN_SLOT: B256 =
    b256!("b53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103");
/// The storage slot of the UpgradeableBeacon contract which defines the
/// implementation for this proxy.
///
/// This is the keccak-256 hash of "eip1967.proxy.beacon" subtracted by 1.
const BEACON_SLOT: B256 =
    b256!("a3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50");

pub trait IErc1967Utils {
    /**
     * @dev Returns the current implementation address.
     */
    fn get_implementation(&self) -> Address;

    /**
     * @dev Performs implementation upgrade with additional setup call if
     * data is nonempty. This function is payable only if the setup call
     * is performed, otherwise `msg.value` is rejected to avoid stuck
     * value in the contract.
     *
     * Emits an {IERC1967-Upgraded} event.
     */
    fn upgrade_to_and_call(&mut self, new_implementation: Address, data: Bytes);

    /**
     * @dev Returns the current admin.
     *
     * TIP: To get this value clients can read directly from the storage
     * slot shown below (specified by ERC-1967) using the https://eth.wiki/json-rpc/API#eth_getstorageat[`eth_getStorageAt`] RPC call.
     * `0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103`
     */
    fn get_admin(&self) -> Address;

    /**
     * @dev Changes the admin of the proxy.
     *
     * Emits an {IERC1967-AdminChanged} event.
     */
    fn change_admin(&mut self, new_admin: Address);

    /**
     * @dev Returns the current beacon.
     */
    fn get_beacon(&self) -> Address;
    /**
     * @dev Change the beacon and trigger a setup call if data is nonempty.
     * This function is payable only if the setup call is performed,
     * otherwise `msg.value` is rejected to avoid stuck value in the
     * contract.
     *
     * Emits an {IERC1967-BeaconUpgraded} event.
     *
     * CAUTION: Invoking this function has no effect on an instance of
     * {BeaconProxy} since v5, since it uses an immutable beacon without
     * looking at the value of the ERC-1967 beacon slot for efficiency.
     */
    fn upgrade_beacon_to_and_call(&mut self, new_beacon: Address, data: Bytes);
}
