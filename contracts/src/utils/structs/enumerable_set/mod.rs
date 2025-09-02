//! Smart contract for managing sets.

pub mod element;

/// Sets have the following properties:
///
/// * Elements are added, removed, and checked for existence in constant
///   time (O(1)).
/// * Elements are enumerated in O(n). No guarantees are made on the
///   ordering.
/// * Set can be cleared (all elements removed) in O(n).
use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, U256};
pub use element::{Accessor, Element};
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
    /// # WARNING
    ///
    /// This function has an unbounded cost that scales with set size.
    /// Developers should keep in mind that using it may render the function
    /// uncallable if the set grows to the point where clearing it consumes too
    /// much gas to fit in a block.
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
    /// Note this implementation's maximum length will technically be `usize`
    /// because that is the type returned by the underlying
    /// [`StorageVec::len`].
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
    /// # WARNING
    ///
    /// This operation will copy the entire storage to memory, which can be
    /// quite expensive. This is designed to mostly be used by view
    /// accessors that are queried without any gas fees. Developers should keep
    /// in mind that this function has an unbounded cost, and using it as
    /// part of a state-changing function may render the function uncallable
    /// if the set grows to a point where copying to memory consumes too much
    /// gas to fit in a block.
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
    macro_rules! impl_enumerable_set_tests {
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


                        // tests idempotency: adding the same element multiple times has no effect
                        #[motsu::test]
                        fn idempotency_add(alice: Address) {
                            proptest!(|(value: $value_type)| {
                                let contract = Contract::<$set_type>::new();

                                let first_add = contract.sender(alice).add(value);
                                prop_assert!(first_add);
                                prop_assert_eq!(contract.sender(alice).length(), U256::from(1));

                                let subsequent_add = contract.sender(alice).add(value);
                                prop_assert!(!subsequent_add);
                                prop_assert_eq!(contract.sender(alice).length(), U256::from(1));
                                prop_assert!(contract.sender(alice).contains(value));
                            });
                        }

                        // tests idempotency: removing a non-existent element has no effect
                        #[motsu::test]
                        fn idempotency_remove(alice: Address) {
                            proptest!(|(value: $value_type)| {
                                let contract = Contract::<$set_type>::new();
                                let remove_result = contract.sender(alice).remove(value);
                                prop_assert!(!remove_result);
                                prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                                prop_assert!(!contract.sender(alice).contains(value));
                            });
                        }

                        // tests commutativity: order of adding elements doesn't affect final set
                        #[motsu::test]
                        fn commutativity_add(alice: Address, bob: Address) {
                            proptest!(|(value1: $value_type, value2: $value_type)| {
                                let contract1 = Contract::<$set_type>::new();
                                let contract2 = Contract::<$set_type>::new();

                                // Add elements in original order to contract1
                                contract1.sender(alice).add(value1);
                                contract1.sender(alice).add(value2);

                                // Reverse elements in contract2
                                contract2.sender(bob).add(value2);
                                contract2.sender(bob).add(value1);

                                // Both sets should be identical
                                prop_assert_eq!(contract1.sender(alice).length(), contract2.sender(bob).length());

                                let values1 = contract1.sender(alice).values().sort();
                                let values2 = contract2.sender(bob).values().sort();
                                prop_assert_eq!(values1, values2);
                            });
                        }


                        // tests associativity: grouping of operations doesn't matter
                        #[motsu::test]
                        fn associativity_operations(alice: Address,bob: Address) {
                            proptest!(|(values1: Vec<$value_type>, values2: Vec<$value_type>)| {
                                if !values1.len() > 0 || !values2.len() > 0 {
                                    return Ok(())
                                }

                                let contract1 = Contract::<$set_type>::new();
                                let contract2 = Contract::<$set_type>::new();
                                for value1 in values1.iter() {
                                    contract1.sender(alice).add(*value1);
                                }
                                for value2 in values2.iter() {
                                    contract2.sender(bob).add(*value2);
                                }

                                for v in contract2.sender(bob).values() {
                                    contract1.sender(alice).add(v);
                                }

                                prop_assert!(values1.len() > 0);
                                let c_values = contract1.sender(alice).values();
                                let all_values: BTreeSet<_> = values1.iter().chain(values2.iter()).collect();
                                let final_values: BTreeSet<_> = c_values.iter().collect();

                                prop_assert_eq!(final_values, all_values);
                            });
                        }


                        // tests identity element: empty set behavior
                        #[motsu::test]
                        fn identity_empty_set(alice: Address) {
                            let contract = Contract::<$set_type>::new();

                            // Empty set properties
                            assert_eq!(contract.sender(alice).length(), U256::ZERO);
                            assert_eq!(contract.sender(alice).values(), Vec::<$value_type>::new());
                            assert_eq!(contract.sender(alice).at(U256::ZERO), None);

                            // Clear on empty set should remain empty
                            contract.sender(alice).clear();
                            assert_eq!(contract.sender(alice).length(), U256::ZERO);
                        }

                        // tests edge case: single element index alignment
                        #[motsu::test]
                        fn single_element_edge_case(alice: Address) {
                            proptest!(|(value: $value_type)| {
                                let contract = Contract::<$set_type>::new();

                                let first_add = contract.sender(alice).add(value);
                                prop_assert!(first_add);
                                prop_assert_eq!(contract.sender(alice).length(), U256::from(1));
                                prop_assert!(contract.sender(alice).at(U256::ZERO).is_some());

                                let was_removed = contract.sender(alice).remove(value);
                                prop_assert!(was_removed);

                                prop_assert!(contract.sender(alice).at(U256::ZERO).is_none());
                                prop_assert!(contract.sender(alice).at(uint!(1_U256)).is_none());

                                contract.sender(alice).clear();

                                prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                            });
                        }

                        // tests inverse/complement: add then remove should restore original state
                        #[motsu::test]
                        fn inverse_add_remove(alice: Address) {
                            proptest!(|(values: Vec<$value_type>)| {

                                let contract = Contract::<$set_type>::new();
                                // Add all unique values
                                let mut unique_values = Vec::new();
                                for value in values.iter() {
                                    if contract.sender(alice).add(*value) {
                                        unique_values.push(*value);
                                    }
                                }

                                // Remove all added values
                                for value in unique_values.iter() {
                                    let was_removed = contract.sender(alice).remove(*value);
                                    prop_assert!(was_removed);
                                }

                                // Should be back to empty state
                                prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                                prop_assert_eq!(contract.sender(alice).values(), Vec::<$value_type>::new());

                                // None of the values should be contained anymore
                                for value in unique_values.iter() {
                                    prop_assert!(!contract.sender(alice).contains(*value));
                                }
                            });
                        }

                        // tests subset relationship: all elements in values() should be contained
                        #[motsu::test]
                        fn subset_values_contained(alice: Address) {
                            proptest!(|(values: Vec<$value_type>)| {
                                let contract = Contract::<$set_type>::new();

                                for value in values.iter() {
                                    contract.sender(alice).add(*value);
                                }

                                let all_values = contract.sender(alice).values();

                                // Every value returned by values() should be contained
                                for value in all_values.iter() {
                                    prop_assert!(contract.sender(alice).contains(*value));
                                }

                                // Length should match
                                prop_assert_eq!(contract.sender(alice).length(), U256::from(all_values.len()));
                            });
                        }

                        // tests cardinality preservation: length should equal unique elements
                        #[motsu::test]
                        fn cardinality_preservation(alice: Address) {
                            proptest!(|(values: Vec<$value_type>)| {

                                let contract = Contract::<$set_type>::new();
                                let mut expected_set = BTreeSet::new();

                                for value in values.iter() {
                                    contract.sender(alice).add(*value);
                                    expected_set.insert(*value);
                                }

                                prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_set.len()));

                                // After operations, cardinality should still match
                                for value in values.iter().take(values.len() / 2) {
                                    contract.sender(alice).remove(*value);
                                    expected_set.remove(value);
                                }

                                prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_set.len()));
                            });
                        }

                        // tests transitivity: if we can enumerate all elements, we should be able to access them by index
                        #[motsu::test]
                        fn transitivity_enumeration(alice: Address) {
                            proptest!(|(values: Vec<$value_type>)| {
                                let contract = Contract::<$set_type>::new();

                                for value in values.iter() {
                                    contract.sender(alice).add(*value);
                                }

                                let length = contract.sender(alice).length();
                                let all_values = contract.sender(alice).values();

                                // Should be able to access each element by index
                                for i in 0..length.try_into().unwrap_or(0) {
                                    let at_result = contract.sender(alice).at(U256::from(i));
                                    prop_assert!(at_result.is_some());

                                    if let Some(value) = at_result {
                                        prop_assert!(all_values.contains(&value));
                                        prop_assert!(contract.sender(alice).contains(value));
                                    }
                                }
                            });
                        }


                        // tests consistency across operations: multiple ways to achieve same state should be equivalent
                        #[motsu::test]
                        fn consistency_multiple_paths(alice: Address, bob: Address) {
                            proptest!(|(values: Vec<$value_type>)| {
                                let contract1 = Contract::<$set_type>::new();
                                let contract2 = Contract::<$set_type>::new();

                                // Path 1: Add all, then remove some, then add them back
                                for value in values.iter() {
                                    contract1.sender(alice).add(*value);
                                }

                                let to_remove: Vec<_> = values.iter().take(values.len() / 2).cloned().collect();
                                for value in to_remove.iter() {
                                    contract1.sender(alice).remove(*value);
                                }

                                for value in to_remove.iter() {
                                    contract1.sender(alice).add(*value);
                                }

                                // Path 2: Just add all values directly
                                for value in values.iter() {
                                    contract2.sender(bob).add(*value);
                                }

                                // Both paths should result in the same set
                                prop_assert_eq!(contract1.sender(alice).length(), contract2.sender(bob).length());

                                let values1 = contract1.sender(alice).values();
                                let values2 = contract2.sender(bob).values();
                                let set1: BTreeSet<_> = values1.into_iter().collect();
                                let set2: BTreeSet<_> = values2.into_iter().collect();
                                prop_assert_eq!(set1, set2);
                            });
                        }
                    }
                )+
            };
        }

    impl_enumerable_set_tests!(
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
