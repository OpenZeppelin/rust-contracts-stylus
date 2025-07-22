//! Smart contract for managing sets.

/// Sets have the following properties:
///
/// * Elements are added, removed, and checked for existence in constant
///   time (O(1)).
/// * Elements are enumerated in O(n). No guarantees are made on the
///   ordering.
/// * Set can be cleared (all elements removed) in O(n).
use alloc::{vec, vec::Vec};

use alloy_primitives::{
    uint, Address, FixedBytes, U128, U16, U256, U32, U64, U8,
};
use stylus_sdk::{
    prelude::*,
    storage::{
        StorageAddress, StorageFixedBytes, StorageKey, StorageMap, StorageType,
        StorageU128, StorageU16, StorageU256, StorageU32, StorageU64,
        StorageU8, StorageVec,
    },
};

/// [`EnumerableSet`] trait for defining sets of primitive types.
///
/// See the `impl_set` macro for the implementation of this trait for many
/// primitive types.
pub trait EnumerableSet<T>
where
    T: StorageKey,
{
    /// Adds a value to a set.
    ///
    /// Returns true if the `value` was added to the set, that is if it was not
    /// already present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to add to the set.
    fn add(&mut self, value: T) -> bool;

    /// Removes a `value` from a set.
    ///
    /// Returns true if the `value` was removed from the set, that is if it was
    /// present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to remove from the set.
    fn remove(&mut self, value: T) -> bool;

    /// Returns true if the `value` is in the set.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `value` - The value to check for in the set.
    fn contains(&self, value: T) -> bool;

    /// Returns the number of values in the set.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    fn length(&self) -> U256;

    /// Returns the value stored at position `index` in the set.
    ///
    /// Note that there are no guarantees on the ordering of values inside the
    /// array, and it may change when more values are added or removed.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `index` - The index of the value to return.
    fn at(&self, index: U256) -> Option<T>;

    /// Returns the entire set in an array.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    fn values(&self) -> Vec<T>;
}

/// Implements the [`EnumerableSet`] trait for the given types.
macro_rules! impl_set {
    ($($name:ident, $skey:ident, $svalue:ident);+ $(;)?) => {
        $(
            /// State of an [`$name`] contract.
            #[storage]
            pub struct $name {
                /// Values in the set.
                values: StorageVec<$svalue>,
                /// Value -> Index of the value in the `values` array.
                positions: StorageMap<$skey, StorageU256>,
            }

            impl EnumerableSet<$skey> for $name {
                fn add(&mut self, value: $skey) -> bool {
                    if self.contains(value) {
                        false
                    } else {
                        self.values.push(value);
                        // The value is stored at length-1, but we add 1 to all indexes
                        // and use [`U256::ZERO`] as a sentinel value.
                        self.positions.setter(value).set(U256::from(self.values.len()));
                        true
                    }
                }

                fn remove(&mut self, value: $skey) -> bool {
                    // We cache the value's position to prevent multiple reads from the same
                    // storage slot.
                    let position = self.positions.get(value);

                    if position.is_zero() {
                        false
                    } else {
                        // To delete an element from the `self.values` array in O(1),
                        // we swap the element to delete with the last one in the array,
                        // and then remove the last element (sometimes called as 'swap and
                        // pop'). This modifies the order of the array, as noted
                        // in [`Self::at`].
                        let one = uint!(1_U256);
                        let value_index = position - one;
                        let last_index = self.length() - one;

                        if value_index != last_index {
                            let last_value = self
                                .values
                                .get(last_index)
                                .expect("element at `last_index` must exist");

                            // Move the `last_value` to the index where the value to delete
                            // is.
                            self.values
                                .setter(value_index)
                                .expect(
                                    "element at `value_index` must exist - is being removed",
                                )
                                .set(last_value);
                            // Update the tracked position of the `last_value` (that was just
                            // moved).
                            self.positions.setter(last_value).set(position);
                        }

                        // Delete the slot where the moved value was stored.
                        self.values.pop();

                        // Delete the tracked position for the deleted slot.
                        self.positions.delete(value);

                        true
                    }
                }

                fn contains(&self, value: $skey) -> bool {
                    !self.positions.get(value).is_zero()
                }

                fn length(&self) -> U256 {
                    U256::from(self.values.len())
                }

                fn at(&self, index: U256) -> Option<$skey> {
                    self.values.get(index)
                }

                fn values(&self) -> Vec<$skey> {
                    let mut values = Vec::new();
                    for idx in 0..self.values.len() {
                        values.push(
                            self.values
                                .get(idx)
                                .expect("element at index: {idx} must exist"),
                        );
                    }
                    values
                }
            }
        )+
    };
}

// use aliases to avoid macro issues with brackets.
type Bytes32 = FixedBytes<32>;
type StorageBytes32 = StorageFixedBytes<32>;

impl_set!(
    EnumerableAddressSet, Address, StorageAddress;
    EnumerableBytes32Set, Bytes32, StorageBytes32;
    EnumerableU8Set, U8, StorageU8;
    EnumerableU16Set, U16, StorageU16;
    EnumerableU32Set, U32, StorageU32;
    EnumerableU64Set, U64, StorageU64;
    EnumerableU128Set, U128, StorageU128;
    EnumerableU256Set, U256, StorageU256;
);

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeSet, vec::Vec};

    use alloy_primitives::{Address, U128, U16, U256, U32, U64, U8};
    use motsu::prelude::Contract;
    use proptest::{prop_assert, prop_assert_eq, proptest};
    use stylus_sdk::prelude::{public, TopLevelStorage};

    use super::*;

    /// Property tests for [`EnumerableSet<T>`] trait.
    ///
    /// The property tests will automatically generate random test cases and
    /// verify that the [`EnumerableSet<T>`] implementations maintain their
    /// properties across all types.
    macro_rules! impl_set_property_tests {
            ($($value_type:ty, $set_type:ty, $test_mod:ident);+ $(;)?) => {
                $(
                    mod $test_mod {
                        use super::*;

                        unsafe impl TopLevelStorage for $set_type {}

                        #[public]
                        impl $set_type {}

                        // verifies that adding values returns correct boolean results and
                        // all added values are contained.
                        #[test]
                        fn prop_add_contains_consistency() {
                            proptest!(|(values: Vec<$value_type>, alice: Address)| {
                                let contract = Contract::<$set_type>::default();
                                let mut expected_values = Vec::new();

                                for value in values.iter() {
                                    let was_added = contract.sender(alice).add(*value);

                                    if !expected_values.contains(value) {
                                        prop_assert!(was_added);
                                        expected_values.push(*value);
                                    } else {
                                        prop_assert!(!was_added);
                                    }

                                    prop_assert!(contract.sender(alice).contains(*value));
                                }

                                prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_values.len()));
                            });
                        }

                        // ensures removal operations work correctly and removed values are
                        // no longer contained.
                        #[test]
                        fn prop_remove_contains_consistency() {
                            proptest!(|(values: Vec<$value_type>, alice: Address)| {
                                let contract = Contract::<$set_type>::default();
                                let mut unique_values = Vec::new();

                                for value in values.iter() {
                                    if !unique_values.contains(value) {
                                        contract.sender(alice).add(*value);
                                        unique_values.push(*value);
                                    }
                                }

                                for value in values.iter() {
                                    let was_removed = contract.sender(alice).remove(*value);

                                    if unique_values.contains(value) {
                                        prop_assert!(was_removed);
                                        unique_values.retain(|&x| x != *value);
                                    } else {
                                        prop_assert!(!was_removed);
                                    }

                                    prop_assert!(!contract.sender(alice).contains(*value));
                                    prop_assert_eq!(contract.sender(alice).length(), U256::from(unique_values.len()));
                                }
                            });
                        }

                        // validates that [`EnumerableSet::<T>::at()`] method returns correct values within bounds and None
                        // for out-of-bounds indices.
                        #[test]
                        fn prop_at_index_bounds() {
                            proptest!(|(values: Vec<$value_type>, alice: Address)| {
                                let contract = Contract::<$set_type>::default();
                                let mut unique_values = Vec::new();

                                for value in values.iter() {
                                    if contract.sender(alice).add(*value) {
                                        unique_values.push(*value);
                                    }
                                }

                                let length = contract.sender(alice).length();
                                prop_assert_eq!(length, U256::from(unique_values.len()));

                                let val: usize = length.try_into().unwrap();
                                for i in 0..val {
                                    let at_result = contract.sender(alice).at(U256::from(i));
                                    prop_assert!(at_result.is_some());

                                    if let Some(value) = at_result {
                                        prop_assert!(unique_values.contains(&value));
                                    }
                                }

                                prop_assert_eq!(contract.sender(alice).at(length), None);
                            });
                        }

                        // confirms that [`EnumerableSet::<T>::values()`] returns exactly the set of added elements.
                        #[test]
                        fn prop_values_completeness() {
                            proptest!(|(values: Vec<$value_type>, alice: Address)| {
                                let contract = Contract::<$set_type>::default();
                                let mut expected_set = BTreeSet::new();

                                for value in values.iter() {
                                    if contract.sender(alice).add(*value) {
                                        expected_set.insert(*value);
                                    }
                                }

                                let returned_values = contract.sender(alice).values();
                                let returned_set: BTreeSet<_> = returned_values.iter().cloned().collect();

                                prop_assert_eq!(returned_set, expected_set.clone());
                                prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_set.len()));
                            });
                        }

                        // tests complex sequences of add/remove operations maintain set semantics.
                        #[test]
                        fn prop_add_remove_sequence_invariants() {
                            proptest!(|(operations: Vec<(bool, $value_type)>, alice: Address)| {
                                let contract = Contract::<$set_type>::default();
                                let mut expected_set = BTreeSet::new();

                                for (is_add, value) in operations.iter() {
                                    if *is_add {
                                        let was_added = contract.sender(alice).add(*value);
                                        let was_new = expected_set.insert(*value);
                                        prop_assert_eq!(was_added, was_new);
                                    } else {
                                        let was_removed = contract.sender(alice).remove(*value);
                                        let was_present = expected_set.remove(value);
                                        prop_assert_eq!(was_removed, was_present);
                                    }

                                    prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_set.len()));

                                    for expected_value in expected_set.iter() {
                                        prop_assert!(contract.sender(alice).contains(*expected_value));
                                    }
                                }
                            });
                        }

                        // verifies correct behavior on empty sets.
                        #[test]
                        fn prop_empty_set_invariants() {
                            proptest!(|(alice: Address, value: $value_type)| {
                                let contract = Contract::<$set_type>::default();

                                prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                                prop_assert!(!contract.sender(alice).contains(value));
                                prop_assert_eq!(contract.sender(alice).at(U256::ZERO), None);
                                prop_assert_eq!(contract.sender(alice).values(), Vec::<$value_type>::new());
                                prop_assert!(!contract.sender(alice).remove(value));
                            });
                        }
                    }
                )+
            };
        }

    impl_set_property_tests!(
        Address, EnumerableAddressSet, address_properties;
        Bytes32, EnumerableBytes32Set, bytes32_properties;
        U8, EnumerableU8Set, u8_properties;
        U16, EnumerableU16Set, u16_properties;
        U32, EnumerableU32Set, u32_properties;
        U64, EnumerableU64Set, u64_properties;
        U128, EnumerableU128Set, u128_properties;
        U256, EnumerableU256Set, u256_properties;
    );
}
