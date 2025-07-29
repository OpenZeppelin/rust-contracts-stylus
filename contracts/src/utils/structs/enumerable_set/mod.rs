//! Smart contract for managing sets.

pub mod generic_size;

/// Sets have the following properties:
///
/// * Elements are added, removed, and checked for existence in constant
///   time (O(1)).
/// * Elements are enumerated in O(n). No guarantees are made on the
///   ordering.
/// * Set can be cleared (all elements removed) in O(n).
use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, U256};
use generic_size::{Accessor, Element};
use stylus_sdk::{
    prelude::*,
    storage::{StorageMap, StorageType, StorageU256, StorageVec},
};

/// State of an [`EnumerableSet`] contract.
#[storage]
pub struct EnumerableSet<T: Element> {
    /// Values in the set.
    values: StorageVec<T::StorageElement>,
    /// Position is the index of the value in the `values` array plus 1.
    /// Position 0 is used to mean a value is not in the set.
    positions: StorageMap<T, StorageU256>,
}

impl<T: Element> EnumerableSet<T> {
    /// Adds a value to a set.
    ///
    /// Returns true if the `value` was added to the set, that is if it was not
    /// already present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to add to the set.
    pub fn add(&mut self, value: T) -> bool {
        if self.contains(value) {
            false
        } else {
            self.values.push(value);

            let position = self.length();
            self.positions.setter(value).set(position);
            true
        }
    }

    /// Removes a `value` from a set.
    ///
    /// Returns true if the `value` was removed from the set, that is if it was
    /// present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to remove from the set.
    #[allow(clippy::missing_panics_doc)]
    pub fn remove(&mut self, value: T) -> bool {
        let position = self.positions.get(value);

        if position.is_zero() {
            false
        } else {
            let one = uint!(1_U256);
            let value_index = position - one;
            let last_index = self.length() - one;

            if value_index != last_index {
                let last_value = self
                    .values
                    .get(last_index)
                    .expect("element at `last_index` must exist");

                self.values
                    .setter(value_index)
                    .expect(
                        "element at `value_index` must exist - is being removed",
                    )
                    .set(last_value);

                self.positions.setter(last_value).set(position);
            }

            self.values.pop();

            self.positions.delete(value);

            true
        }
    }

    /// Remove all values from a set.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    #[allow(clippy::missing_panics_doc)]
    pub fn clear(&mut self) {
        for idx in 0..self.values.len() {
            let v = self
                .values
                .get(idx)
                .expect("element at index: {idx} must exist");
            self.positions.delete(v);
        }

        self.values.erase();
    }

    /// Returns true if the `value` is in the set.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `value` - The value to check for in the set.
    pub fn contains(&self, value: T) -> bool {
        !self.positions.get(value).is_zero()
    }

    /// Returns the number of values in the set.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    pub fn length(&self) -> U256 {
        U256::from(self.values.len())
    }

    /// Returns the value stored at position `index` in the set.
    ///
    /// Note that there are no guarantees on the ordering of values inside the
    /// array, and it may change when more values are added or removed.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `index` - The index of the value to return.
    pub fn at(&self, index: U256) -> Option<T> {
        self.values.get(index)
    }

    /// Returns the entire set in an array.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    #[allow(clippy::missing_panics_doc)]
    pub fn values(&self) -> Vec<T> {
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, B256, U128, U16, U256, U32, U64, U8};

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
                        use motsu::prelude::Contract;
                        use alloc::collections::BTreeSet;
                        use stylus_sdk::prelude::TopLevelStorage;
                        use alloy_primitives::private::proptest::{prop_assert, prop_assert_eq, proptest};

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

                        // verifies the `clear` function properly empties the set.
                        #[test]
                        fn prop_clear_empties_set() {
                            proptest!(|(values: Vec<$value_type>, alice: Address)| {
                                let contract = Contract::<$set_type>::default();
                                let mut unique_values = Vec::new();

                                for value in values.iter() {
                                    if contract.sender(alice).add(*value) {
                                        unique_values.push(*value);
                                    }
                                }

                                let length_before = contract.sender(alice).length();
                                prop_assert_eq!(length_before, U256::from(unique_values.len()));

                                contract.sender(alice).clear();

                                prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                                prop_assert_eq!(contract.sender(alice).values(), Vec::<$value_type>::new());

                                for value in unique_values.iter() {
                                    prop_assert!(!contract.sender(alice).contains(*value));
                                }

                                prop_assert_eq!(contract.sender(alice).at(U256::ZERO), None);
                            });
                        }
                    }
                )+
            };
        }

    impl_set_property_tests!(
        Address, EnumerableSet::<Address>, address_properties;
        B256, EnumerableSet::<B256>, b256_properties;
        U8, EnumerableSet::<U8>, u8_properties;
        U16, EnumerableSet::<U16>, u16_properties;
        U32, EnumerableSet::<U32>, u32_properties;
        U64, EnumerableSet::<U64>, u64_properties;
        U128, EnumerableSet::<U128>, u128_properties;
        U256, EnumerableSet::<U256>, u256_properties;
    );
}
