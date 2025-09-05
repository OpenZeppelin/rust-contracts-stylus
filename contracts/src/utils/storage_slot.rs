//! Helper for reading and writing primitive types to specific storage slots.
use alloy_primitives::U256;
use stylus_sdk::{host::VM, prelude::*};

const SLOT_BYTE_SPACE: u8 = 32;

/// Helper for reading and writing primitive types to specific storage slots.
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
/// extern crate alloc;
///
/// use alloc::{vec, vec::Vec};
/// use alloy_primitives::{b256, Address, B256};
/// use openzeppelin_stylus::utils::storage_slot::StorageSlot;
/// use stylus_sdk::{storage::StorageAddress, prelude::*};
///
/// const IMPLEMENTATION_SLOT: B256 = b256!("0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");
///
/// #[storage]
/// #[entrypoint]
/// pub struct Erc1967;
///
/// #[public]
/// impl Erc1967 {
///     fn get_implementation(&self) -> Address {
///         return StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).get();
///     }
///
///     fn set_implementation(&mut self, new_implementation: Address) {
///         assert!(new_implementation.has_code());
///         StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).set(new_implementation);
///     }
/// }
/// ```
pub struct StorageSlot;

impl StorageSlot {
    /// Returns a [`StorageType`] located at `slot`.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot to get the address from.
    #[must_use]
    pub fn get_slot<ST: StorageType>(slot: impl Into<U256>) -> ST {
        // TODO: Remove this once we have a proper way to inject the host for
        // custom storage slot access.
        // This has been implemented on Stylus SDK 0.10.0.

        // Priority order:
        // 1. If wasm32 target -> always use tuple syntax (highest priority).
        // 2. If reentrant feature enabled (on non-wasm32) -> use struct syntax.
        // 3. If non-wasm32 without export-abi -> use struct syntax.
        // 4. Everything else -> use tuple syntax.

        #[cfg(all(
            not(target_arch = "wasm32"),
            any(feature = "reentrant", not(feature = "export-abi"))
        ))]
        let host =
            VM { host: alloc::boxed::Box::new(stylus_sdk::host::WasmVM {}) };

        #[cfg(not(all(
            not(target_arch = "wasm32"),
            any(feature = "reentrant", not(feature = "export-abi"))
        )))]
        let host = VM(stylus_sdk::host::WasmVM {});

        // SAFETY: Truncation is safe here because ST::SLOT_BYTES is never
        // larger than 32, so the subtraction cannot underflow and the
        // cast is always valid.
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            ST::new(slot.into(), SLOT_BYTE_SPACE - ST::SLOT_BYTES as u8, host)
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unused_self)]

    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::*;
    use stylus_sdk::storage::StorageAddress;

    use super::*;

    const IMPLEMENTATION_SLOT: U256 = uint!(12345_U256);

    #[storage]
    #[entrypoint]
    pub struct Erc1967 {
        address: StorageAddress,
    }

    #[public]
    impl Erc1967 {
        fn get_implementation(&self) -> Address {
            StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT).get()
        }

        fn set_implementation(&self, new_implementation: Address) {
            StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT)
                .set(new_implementation);
        }

        fn get_address(&self) -> Address {
            self.address.get()
        }

        fn set_address(&mut self, new_address: Address) {
            self.address.set(new_address);
        }

        fn get_address_at_zero_slot(&self) -> Address {
            StorageSlot::get_slot::<StorageAddress>(U256::ZERO).get()
        }

        fn set_address_at_zero_slot(&mut self, new_address: Address) {
            StorageSlot::get_slot::<StorageAddress>(U256::ZERO)
                .set(new_address);
        }
    }

    #[motsu::test]
    fn test_storage_slot(
        contract: Contract<Erc1967>,
        alice: Address,
        impl_address: Address,
    ) {
        let implementation = contract.sender(alice).get_implementation();
        assert_eq!(implementation, Address::ZERO);

        contract.sender(alice).set_implementation(impl_address);

        let implementation = contract.sender(alice).get_implementation();
        assert_eq!(implementation, impl_address);
        let address = contract.sender(alice).get_address_at_zero_slot();
        assert_eq!(address, Address::ZERO);
        let address = contract.sender(alice).get_address();
        assert_eq!(address, Address::ZERO);

        contract.sender(alice).set_address(impl_address);

        let implementation = contract.sender(alice).get_implementation();
        assert_eq!(implementation, impl_address);
        let address = contract.sender(alice).get_address_at_zero_slot();
        assert_eq!(address, impl_address);
        let address = contract.sender(alice).get_address();
        assert_eq!(address, impl_address);

        contract.sender(alice).set_address_at_zero_slot(alice);

        let implementation = contract.sender(alice).get_implementation();
        assert_eq!(implementation, impl_address);
        let address = contract.sender(alice).get_address_at_zero_slot();
        assert_eq!(address, alice);
        let address = contract.sender(alice).get_address();
        assert_eq!(address, alice);
    }
}
