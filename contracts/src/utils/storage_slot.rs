use alloc::{boxed::Box, vec, vec::Vec};

use alloy_primitives::{Address, B256};
use stylus_sdk::{host::VM, prelude::*, storage::StorageAddress};

/// Trait for reading and writing primitive types to specific storage slots.
///
/// Storage slots are often used to avoid storage conflict when dealing with
/// upgradeable contracts. This library helps with reading and writing to such
/// slots without the need for low-level operations.
///
/// The functions in this library return appropriate storage types that contain
/// a `value` member that can be used to read or write.
///
/// Example usage to set ERC-1967 implementation slot:
///
/// ```rust
/// const IMPLEMENTATION_SLOT: B256 = b256!("0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");
///
/// #[storage]
/// #[entrypoint]
/// pub struct Erc1967;
///
/// #[public]
/// impl Erc1967 {
///     fn get_implementation(&self) -> Address {
///         return StorageAddress::get_slot(IMPLEMENTATION_SLOT).get();
///     }
///
///     fn set_implementation(&self, new_implementation: Address) {
///         assert!(Address::has_code(new_implementation));
///         StorageAddress::get_slot(IMPLEMENTATION_SLOT).set(new_implementation);
///     }
/// }
/// ```
pub trait StorageSlotType: StorageType + HostAccess {
    /// Returns a [`StorageType`] located at `slot`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `slot` - The slot to get the address from.
    fn get_slot<ST: StorageType>(&self, slot: B256) -> ST {
        #[cfg(feature = "stylus-test")]
        let host = VM { host: Box::new(stylus_sdk::host::WasmVM {}) };
        #[cfg(not(feature = "stylus-test"))]
        let host = VM(stylus_sdk::host::WasmVM {});
        unsafe { ST::new(slot.into(), 0, host) }
    }
}

impl<T: StorageType + HostAccess> StorageSlotType for T {}
