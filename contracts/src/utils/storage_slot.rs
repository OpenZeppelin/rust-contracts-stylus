use alloc::boxed::Box;

use stylus_sdk::{alloy_primitives::B256, host::VM, prelude::*};

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
///         return self.get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).get();
///     }
///
///     fn set_implementation(&self, new_implementation: Address) {
///         assert!(Address::has_code(new_implementation));
///         self.get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).set(new_implementation);
///     }
/// }
/// ```
pub trait StorageSlot: StorageType + HostAccess {
    /// Returns a [`StorageType`] located at `slot`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `slot` - The slot to get the address from.
    fn get_slot<ST: StorageType>(&self, slot: B256) -> ST {
        unsafe {
            ST::new(
                slot.into(),
                0,
                VM { host: Box::new(stylus_sdk::host::WasmVM {}) },
            )
        }
    }
}

impl<T: StorageType + HostAccess> StorageSlot for T {}
