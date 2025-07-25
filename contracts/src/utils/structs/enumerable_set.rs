//! Smart contract for managing sets.

/// Sets have the following properties:
///
/// * Elements are added, removed, and checked for existence in constant
///   time (O(1)).
/// * Elements are enumerated in O(n). No guarantees are made on the
///   ordering.
/// * Set can be cleared (all elements removed) in O(n).
use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, B256, U128, U16, U256, U32, U64, U8};
use stylus_sdk::{
    prelude::*,
    storage::{
        StorageAddress, StorageB256, StorageKey, StorageMap, StorageType,
        StorageU128, StorageU16, StorageU256, StorageU32, StorageU64,
        StorageU8, StorageVec,
    },
};

/// [`EnumerableSet`] trait for defining sets of primitive types.
///
/// See the [`crate::impl_enumerable_set`] macro for the implementation of this
/// trait for many primitive types.
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
    ///
    /// # Panics
    ///
    /// Panics if an index does not exist for the given `value`.
    fn remove(&mut self, value: T) -> bool;

    /// Remove all values from a set.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    ///
    /// # Panics
    ///
    /// Panics if an index does not exist for the given `value`.
    fn clear(&mut self);

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
    ///
    /// # Panics
    ///
    /// Panics if an index does not exist for the given `value`.
    fn values(&self) -> Vec<T>;
}

/// Implements the [`EnumerableSet`] trait for the given types.
///
/// # Arguments
///
/// * `name`   - The name of the implementation
/// * `skey`   - The storage key type associated with the set. This type must
///   implement Stylus SDK StorageKey
/// * `svalue` - The sets storage type
#[macro_export]
macro_rules! impl_enumerable_set {
    ($($name:ident, $skey:ident, $svalue:ident);+ $(;)?) => {
        $(
            #[doc = concat!("State of an [`",stringify!($name),"`] contract.")]
            #[storage]
            pub struct $name {
                // Values in the set.
                values: StorageVec<$svalue>,
                // Position is the index of the value in the `values` array plus 1.
                // Position 0 is used to mean a value is not in the set.
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
                        let position = self.length();
                        self.positions.setter(value).set(position);
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

                fn clear(&mut self) {
                    for idx in 0..self.values.len() {
                        // get values to delete from map
                        let v = self.values.get(idx).expect("element at index: {idx} must exist");
                        self.positions.delete(v);
                    }
                    // clear all the values
                    self.values.erase();
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

impl_enumerable_set!(
    EnumerableAddressSet, Address, StorageAddress;
    EnumerableB256Set, B256, StorageB256;
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

    use alloy_primitives::{
        private::proptest::{prop_assert, prop_assert_eq, proptest},
        Address, U128, U16, U256, U32, U64, U8,
    };
    use motsu::prelude::*;
    use stylus_sdk::prelude::{public, TopLevelStorage};

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
                                if !values1.len() > 0 || values2.len() > 0 {
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
        Address, EnumerableAddressSet, address_properties;
        B256, EnumerableB256Set, b256_properties;
        U8, EnumerableU8Set, u8_properties;
        U16, EnumerableU16Set, u16_properties;
        U32, EnumerableU32Set, u32_properties;
        U64, EnumerableU64Set, u64_properties;
        U128, EnumerableU128Set, u128_properties;
        U256, EnumerableU256Set, u256_properties;
    );
}
