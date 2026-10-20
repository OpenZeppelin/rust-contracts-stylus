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
/// use openzeppelin_stylus::utils::{address::AddressExt, storage_slot::StorageSlot};
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
///         return StorageSlot::get_slot::<StorageAddress>(self, IMPLEMENTATION_SLOT).get();
///     }
///
///     fn set_implementation(&mut self, new_implementation: Address) {
///         assert!(new_implementation.has_code(self.vm()));
///         StorageSlot::get_slot::<StorageAddress>(self, IMPLEMENTATION_SLOT).set(new_implementation);
///     }
/// }
/// ```
pub struct StorageSlot;

impl StorageSlot {
    /// Returns a storage accessor of type `ST` bound to `slot` in the storage
    /// of the contract that `context` belongs to.
    ///
    /// # Arguments
    ///
    /// * `context` - Any `#[storage]` struct of the contract, used to reach the
    ///   host VM.
    /// * `slot` - The storage slot to access.
    #[must_use]
    pub fn get_slot<ST: StorageType>(
        context: &impl HostAccess<Host = VM>,
        slot: impl Into<U256>,
    ) -> ST {
        #[allow(clippy::cast_possible_truncation)]
        unsafe {
            ST::new(
                slot.into(),
                SLOT_BYTE_SPACE - ST::SLOT_BYTES as u8,
                context.vm().clone(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address, U256};
    use stylus_sdk::storage::StorageAddress;
    use testing::prelude::*;

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
            StorageSlot::get_slot::<StorageAddress>(self, IMPLEMENTATION_SLOT)
                .get()
        }

        fn set_implementation(&self, new_implementation: Address) {
            StorageSlot::get_slot::<StorageAddress>(self, IMPLEMENTATION_SLOT)
                .set(new_implementation);
        }

        fn get_address(&self) -> Address {
            self.address.get()
        }

        fn set_address(&mut self, new_address: Address) {
            self.address.set(new_address);
        }

        fn get_address_at_zero_slot(&self) -> Address {
            StorageSlot::get_slot::<StorageAddress>(self, U256::ZERO).get()
        }

        fn set_address_at_zero_slot(&mut self, new_address: Address) {
            StorageSlot::get_slot::<StorageAddress>(self, U256::ZERO)
                .set(new_address);
        }
    }

    #[testing::test]
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
