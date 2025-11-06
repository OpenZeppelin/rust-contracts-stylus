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

use alloy_primitives::U256;
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
            let value_index = position - U256::ONE;
            let last_index = self.length() - U256::ONE;

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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

    /// Returns a slice of the set in an array.
    ///
    /// This function allows querying subsets of large sets to avoid
    /// out-of-gas scenarios when the full set would be too expensive
    /// to return in a single call.
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
    /// * `start` - The starting index (inclusive).
    /// * `end` - The ending index (exclusive).
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn values_slice(&self, start: U256, end: U256) -> Vec<T> {
        let len = self.length();

        // clamp parameters to valid ranges as in Solidity implementation: <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/448538259f1feeb24f3e3201115d70818ba876cb/contracts/utils/structs/EnumerableSet.sol#L189>.
        let end = core::cmp::min(end, len);
        let start = core::cmp::min(start, end);

        // Uses optimized native iteration for typical cases where indices
        // fit within [`usize`] bounds. Falls back to [`U256`] arithmetic for
        // extremely large indices (theoretical edge case).
        if let (Ok(start_idx), Ok(end_idx)) =
            (usize::try_from(start), usize::try_from(end))
        {
            // fast path: use native [`usize`] iteration with iterator
            // chains.
            (start_idx..end_idx)
                .map(|idx| {
                    self.at(U256::from(idx))
                        .expect("element at index: {idx} must exist")
                })
                .collect()
        } else {
            // slow path: pure [`U256`] arithmetic for extremely large indices.
            let mut result = Vec::new();
            let mut current = start;

            while current < end {
                let value = self
                    .at(current)
                    .expect("element at index: {current} must exist");
                result.push(value);

                current += U256::ONE;
            }

            result
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use alloy_primitives::{
        uint, Address, B256, U128, U16, U256, U32, U64, U8,
    };

    use super::*;

    /// Property tests for [`EnumerableSet<T>`] trait.
    ///
    /// The property tests will automatically generate random test cases and
    /// verify that the [`EnumerableSet<T>`] implementations maintain their
    /// properties across all types.
    macro_rules! impl_enumerable_set_tests {
        ($($value_type:ty, $test_mod:ident);+ $(;)?) => {
            $(
                mod $test_mod {
                    use super::*;
                    use motsu::prelude::Contract;
                    use alloc::collections::BTreeSet;
                    use stylus_sdk::prelude::TopLevelStorage;
                    use alloy_primitives::private::proptest::{prop_assert, prop_assert_eq, proptest};

                    #[storage]
                    struct TestEnumerableSet {
                        set: EnumerableSet<$value_type>
                    }

                    unsafe impl TopLevelStorage for TestEnumerableSet {}

                    #[public]
                    impl TestEnumerableSet {
                        fn add(&mut self, value: $value_type) -> bool {
                            self.set.add(value)
                        }

                        fn remove(&mut self, value: $value_type) -> bool {
                            self.set.remove(value)
                        }

                        fn contains(&self, value: $value_type) -> bool {
                            self.set.contains(value)
                        }

                        fn length(&self) -> U256 {
                            self.set.length()
                        }

                        fn at(&self, index: U256) -> Result<$value_type, Vec<u8>> {
                            self.set.at(index).ok_or(b"Index out of bounds".to_vec())
                        }

                        fn values(&self) -> Vec<$value_type> {
                            self.set.values()
                        }

                        fn values_slice(&self, start: U256, end: U256) -> Vec<$value_type> {
                            self.set.values_slice(start, end)
                        }

                        fn clear(&mut self) {
                            self.set.clear()
                        }
                    }

                    #[motsu::test]
                    fn idempotency_add(alice: Address) {
                        proptest!(|(value: $value_type)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            let first_add = contract.sender(alice).add(value);
                            prop_assert!(first_add);
                            prop_assert_eq!(contract.sender(alice).length(), U256::ONE);

                            let subsequent_add = contract.sender(alice).add(value);
                            prop_assert!(!subsequent_add);
                            prop_assert_eq!(contract.sender(alice).length(), U256::ONE);
                            prop_assert!(contract.sender(alice).contains(value));
                        });
                    }

                    #[motsu::test]
                    fn idempotency_remove(alice: Address) {
                        proptest!(|(value: $value_type)| {
                            let contract = Contract::<TestEnumerableSet>::new();
                            let remove_result = contract.sender(alice).remove(value);
                            prop_assert!(!remove_result);
                            prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                            prop_assert!(!contract.sender(alice).contains(value));
                        });
                    }

                    #[motsu::test]
                    fn commutativity_add(alice: Address, bob: Address) {
                        proptest!(|(value1: $value_type, value2: $value_type)| {
                            let contract1 = Contract::<TestEnumerableSet>::new();
                            let contract2 = Contract::<TestEnumerableSet>::new();

                            contract1.sender(alice).add(value1);
                            contract1.sender(alice).add(value2);

                            contract2.sender(bob).add(value2);
                            contract2.sender(bob).add(value1);

                            prop_assert_eq!(contract1.sender(alice).length(), contract2.sender(bob).length());

                            let mut values1 = contract1.sender(alice).values();
                            let mut values2 = contract2.sender(bob).values();
                            values1.sort();
                            values2.sort();
                            prop_assert_eq!(values1, values2);
                        });
                    }

                    #[motsu::test]
                    fn associativity_operations(alice: Address, bob: Address) {
                        proptest!(|(values1: Vec<$value_type>, values2: Vec<$value_type>)| {
                            if values1.is_empty() || values2.is_empty() {
                                return Ok(());
                            }

                            let contract1 = Contract::<TestEnumerableSet>::new();
                            let contract2 = Contract::<TestEnumerableSet>::new();

                            for value1 in &values1 {
                                contract1.sender(alice).add(*value1);
                            }
                            for value2 in &values2 {
                                contract2.sender(bob).add(*value2);
                            }

                            for v in contract2.sender(bob).values() {
                                contract1.sender(alice).add(v);
                            }

                            let c_values = contract1.sender(alice).values();
                            let all_values: BTreeSet<_> = values1.iter().chain(values2.iter()).collect();
                            let final_values: BTreeSet<_> = c_values.iter().collect();

                            prop_assert_eq!(final_values, all_values);
                        });
                    }

                    #[motsu::test]
                    fn identity_empty_set(alice: Address) {
                        let contract = Contract::<TestEnumerableSet>::new();

                        assert_eq!(contract.sender(alice).length(), U256::ZERO);
                        assert_eq!(contract.sender(alice).values(), Vec::<$value_type>::new());
                        assert!(contract.sender(alice).at(U256::ZERO).is_err());

                        contract.sender(alice).clear();
                        assert_eq!(contract.sender(alice).length(), U256::ZERO);
                    }

                    #[motsu::test]
                    fn single_element_edge_case(alice: Address) {
                        proptest!(|(value: $value_type)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            let first_add = contract.sender(alice).add(value);
                            prop_assert!(first_add);
                            prop_assert_eq!(contract.sender(alice).length(), U256::ONE);
                            prop_assert!(contract.sender(alice).at(U256::ZERO).is_ok());

                            let was_removed = contract.sender(alice).remove(value);
                            prop_assert!(was_removed);

                            prop_assert!(contract.sender(alice).at(U256::ZERO).is_err());
                            prop_assert!(contract.sender(alice).at(U256::ONE).is_err());

                            contract.sender(alice).clear();
                            prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                        });
                    }

                    #[motsu::test]
                    fn inverse_add_remove(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            let mut unique_values = Vec::new();
                            for value in &values {
                                if contract.sender(alice).add(*value) {
                                    unique_values.push(*value);
                                }
                            }

                            for value in &unique_values {
                                let was_removed = contract.sender(alice).remove(*value);
                                prop_assert!(was_removed);
                            }

                            prop_assert_eq!(contract.sender(alice).length(), U256::ZERO);
                            prop_assert_eq!(contract.sender(alice).values(), Vec::<$value_type>::new());

                            for value in &unique_values {
                                prop_assert!(!contract.sender(alice).contains(*value));
                            }
                        });
                    }

                    #[motsu::test]
                    fn subset_values_contained(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            for value in &values {
                                contract.sender(alice).add(*value);
                            }

                            let all_values = contract.sender(alice).values();

                            for value in &all_values {
                                prop_assert!(contract.sender(alice).contains(*value));
                            }

                            prop_assert_eq!(contract.sender(alice).length(), U256::from(all_values.len()));
                        });
                    }

                    #[motsu::test]
                    fn cardinality_preservation(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();
                            let mut expected_set = BTreeSet::new();

                            for value in &values {
                                contract.sender(alice).add(*value);
                                expected_set.insert(*value);
                            }

                            prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_set.len()));

                            for value in values.iter().take(values.len() / 2) {
                                contract.sender(alice).remove(*value);
                                expected_set.remove(value);
                            }

                            prop_assert_eq!(contract.sender(alice).length(), U256::from(expected_set.len()));
                        });
                    }

                    #[motsu::test]
                    fn transitivity_enumeration(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            for value in &values {
                                contract.sender(alice).add(*value);
                            }

                            let length = contract.sender(alice).length();
                            let all_values = contract.sender(alice).values();

                            for i in 0..length.try_into().unwrap_or(0) {
                                let at_result = contract.sender(alice).at(U256::from(i));
                                prop_assert!(at_result.is_ok());

                                if let Ok(value) = at_result {
                                    prop_assert!(all_values.contains(&value));
                                    prop_assert!(contract.sender(alice).contains(value));
                                }
                            }
                        });
                    }

                    #[motsu::test]
                    fn consistency_multiple_paths(alice: Address, bob: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract1 = Contract::<TestEnumerableSet>::new();
                            let contract2 = Contract::<TestEnumerableSet>::new();

                            for value in &values {
                                contract1.sender(alice).add(*value);
                            }

                            let to_remove: Vec<_> = values.iter().take(values.len() / 2).cloned().collect();
                            for value in &to_remove {
                                contract1.sender(alice).remove(*value);
                            }

                            for value in &to_remove {
                                contract1.sender(alice).add(*value);
                            }

                            for value in &values {
                                contract2.sender(bob).add(*value);
                            }

                            prop_assert_eq!(contract1.sender(alice).length(), contract2.sender(bob).length());

                            let values1 = contract1.sender(alice).values();
                            let values2 = contract2.sender(bob).values();
                            let set1: BTreeSet<_> = values1.into_iter().collect();
                            let set2: BTreeSet<_> = values2.into_iter().collect();
                            prop_assert_eq!(set1, set2);

                            contract1.sender(alice).clear();
                            contract2.sender(bob).clear();

                            prop_assert_eq!(contract1.sender(alice).length(), U256::ZERO);
                            prop_assert_eq!(contract2.sender(bob).length(), U256::ZERO);
                        });
                    }

                    // tests slice_consistency: `values_slice()` should be consistent with `values()`.
                    #[motsu::test]
                    fn slice_consistency(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            for value in &values {
                                contract.sender(alice).add(*value);
                            }

                            let len = contract.sender(alice).length();
                            let all_values = contract.sender(alice).values();
                            let full_slice = contract.sender(alice).values_slice(U256::ZERO, len);

                            let set1: BTreeSet<_> = all_values.into_iter().collect();
                            let set2: BTreeSet<_> = full_slice.into_iter().collect();
                            prop_assert_eq!(set1, set2);
                        });
                    }

                    // tests slice_bounds_safety: invalid parameters should return empty safely
                    #[motsu::test]
                    fn slice_bounds_safety(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            for value in &values {
                                contract.sender(alice).add(*value);
                            }

                            let len = contract.sender(alice).length();

                            // Test invalid ranges (fast path)
                            let empty1 = contract.sender(alice).values_slice(uint!(10_U256), uint!(5_U256));
                            prop_assert_eq!(empty1.len(), 0);

                            // Test out of bounds (fast path)
                            let empty2 = contract.sender(alice).values_slice(len + U256::ONE, len + uint!(10_U256));
                            prop_assert_eq!(empty2.len(), 0);
                        });
                    }

                    // tests slice_u256_path: explicitly test U256 arithmetic path
                    #[motsu::test]
                    fn slice_u256_path(alice: Address) {
                        let contract = Contract::<TestEnumerableSet>::new();

                        // Add a test value
                        let test_value = <$value_type>::default();
                        contract.sender(alice).add(test_value);

                        // Force U256 path with indices beyond usize::MAX
                        let huge_start = U256::from(usize::MAX) + U256::ONE;
                        let huge_end = huge_start + uint!(100_U256);

                        // This should use the slow U256 path and return empty (out of bounds)
                        let empty_slice = contract.sender(alice).values_slice(huge_start, huge_end);
                        assert_eq!(empty_slice.len(), 0);

                        // Test U256 path with start=0 but end beyond usize::MAX
                        let mixed_slice = contract.sender(alice).values_slice(U256::ZERO, huge_start);
                        let all_values = contract.sender(alice).values();

                        // Should return all values via slow path when end is clamped
                        let set1: BTreeSet<_> = mixed_slice.into_iter().collect();
                        let set2: BTreeSet<_> = all_values.into_iter().collect();
                        assert_eq!(set1, set2);
                    }

                    // tests slice_usize_boundary: test exact boundary between fast/slow paths
                    #[motsu::test]
                    fn slice_usize_boundary(alice: Address) {
                        proptest!(|(values: Vec<$value_type>)| {
                            let contract = Contract::<TestEnumerableSet>::new();

                            for value in &values {
                                contract.sender(alice).add(*value);
                            }

                            let len = contract.sender(alice).length();

                            // Test exactly at usize::MAX boundary (should use fast path)
                            let boundary_end = core::cmp::min(len, U256::from(usize::MAX));
                            let boundary_slice = contract.sender(alice).values_slice(U256::ZERO, boundary_end);

                            // Should produce valid results via fast path
                            for value in &boundary_slice {
                                prop_assert!(contract.sender(alice).contains(*value));
                            }

                            // Test one beyond usize::MAX (should use slow path)
                            let beyond_boundary = U256::from(usize::MAX) + U256::ONE;
                            let beyond_slice = contract.sender(alice).values_slice(U256::ZERO, beyond_boundary);

                            // Should clamp to actual length and return all values via slow path
                            let all_values = contract.sender(alice).values();
                            let set1: BTreeSet<_> = beyond_slice.into_iter().collect();
                            let set2: BTreeSet<_> = all_values.into_iter().collect();
                            prop_assert_eq!(set1, set2);
                        });
                    }
                }
            )+
        };
    }

    impl_enumerable_set_tests!(
        Address, address_properties;
        B256, b256_properties;
        U8, u8_properties;
        U16, u16_properties;
        U32, u32_properties;
        U64, u64_properties;
        U128, u128_properties;
        U256, u256_properties;
    );
}
