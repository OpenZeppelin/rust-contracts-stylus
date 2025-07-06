//! Smart contract for managing sets of [`Address`] type.

/// Sets have the following properties:
///
/// * Elements are added, removed, and checked for existence in constant
///   time (O(1)).
/// * Elements are enumerated in O(n). No guarantees are made on the
///   ordering.
/// * Set can be cleared (all elements removed) in O(n).
use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, FixedBytes, U256};
use stylus_sdk::{
    prelude::*,
    storage::{
        StorageAddress, StorageFixedBytes, StorageKey, StorageMap, StorageType,
        StorageU256, StorageVec,
    },
};

/// EnumarableSet trait for defining new sets.
///
/// This trait is automatically implemented if using the `impl_set` macro.
pub trait EnumerableSet<K, V>
where
    K: StorageKey,
    V: StorageType,
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
    fn add(&mut self, value: K) -> bool;

    /// Removes a `value` from a set.
    ///
    /// Returns true if the `value` was removed from the set, that is if it was
    /// present.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the set's state.
    /// * `value` - The value to remove from the set.
    fn remove(&mut self, value: K) -> bool;

    /// Returns true if the `value` is in the set.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    /// * `value` - The value to check for in the set.
    fn contains(&self, value: K) -> bool;

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
    fn at(&self, index: U256) -> Option<K>;

    /// Returns the entire set in an array.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the set's state.
    fn values(&self) -> Vec<K>;
}

///
macro_rules! impl_set {
    ($($name:ident $key:ident $value:ident)+) => {
        $(
            impl EnumerableSet<$key, $value> for $name {
                fn add(&mut self, value: $key) -> bool {
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

                fn remove(&mut self, value: $key) -> bool {
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
                            // Update the tracked position of the lastValue (that was just
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

                fn contains(&self, value: $key) -> bool {
                    !self.positions.get(value).is_zero()
                }

                fn length(&self) -> U256 {
                    U256::from(self.values.len())
                }

                fn at(&self, index: U256) -> Option<$key> {
                    self.values.get(index)
                }

                fn values(&self) -> Vec<$key> {
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

/// State of an [`EnumerableAddressSet`] contract.
#[storage]
pub struct EnumerableAddressSet {
    /// Values in the set.
    values: StorageVec<StorageAddress>,
    /// Value -> Index of the value in the `values` array.
    positions: StorageMap<Address, StorageU256>,
}
impl_set!(EnumerableAddressSet Address StorageAddress);

/// Sets to add: Uint

#[storage]
pub struct EmumerableBytes32Set {
    /// Values in the set.
    values: StorageVec<StorageFixedBytes<32>>,
    /// Value -> Index of the value in the `values` array.
    positions: StorageMap<FixedBytes<32>, StorageU256>,
}
// use alias to avoid macro issues with brackets
type Bytes32 = FixedBytes<32>;
type StorageBytes32 = StorageFixedBytes<32>;
impl_set!(EmumerableBytes32Set Bytes32 StorageBytes32);

/*
#[storage]
pub struct EmumerableStringSet {
    /// Values in the set.
    values: StorageVec<StorageString>,
    /// Value -> Index of the value in the `values` array.
    positions: StorageMap<String, StorageU256>,
}
impl_set!(EmumerableStringSet String StorageString);
*/

macro_rules! impl_uint_sets {
    ($($uint:ident $int:ident)+) => {
        $(
            #[storage]
            pub struct EmumerableBytes32Set {
                /// Values in the set.
                values: StorageVec<StorageFixedBytes<32>>,
                /// Value -> Index of the value in the `values` array.
                positions: StorageMap<FixedBytes<32>, StorageU256>,
            }

        )+
    };
}

#[cfg(test)]
mod tests {
    use motsu::prelude::Contract;

    use super::*;

    unsafe impl TopLevelStorage for EnumerableAddressSet {}

    #[public]
    impl EnumerableAddressSet {}

    #[motsu::test]
    fn add_multiple_values(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(contract.sender(alice).add(bob));
        assert!(contract.sender(alice).add(charlie));
        assert!(contract.sender(alice).contains(alice));
        assert!(contract.sender(alice).contains(bob));
        assert!(contract.sender(alice).contains(charlie));
        assert_eq!(contract.sender(alice).length(), uint!(3_U256));
        assert_eq!(contract.sender(alice).values(), [alice, bob, charlie]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(alice));
        assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(bob));
        assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(charlie));
        assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
    }

    #[motsu::test]
    fn does_not_duplicate_values(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(!contract.sender(alice).add(alice));
        assert!(contract.sender(alice).contains(alice));
        assert_eq!(contract.sender(alice).length(), uint!(1_U256));
        assert_eq!(contract.sender(alice).values(), [alice]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(alice));
    }

    #[motsu::test]
    fn removes_first_value(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(contract.sender(alice).add(bob));
        assert!(contract.sender(alice).add(charlie));
        assert!(contract.sender(alice).remove(alice));
        assert!(!contract.sender(alice).contains(alice));
        assert!(contract.sender(alice).contains(bob));
        assert!(contract.sender(alice).contains(charlie));
        assert_eq!(contract.sender(alice).length(), uint!(2_U256));
        assert_eq!(contract.sender(alice).values(), [charlie, bob]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(charlie));
        assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(bob));
        assert_eq!(contract.sender(alice).at(uint!(2_U256)), None);
    }

    #[motsu::test]
    fn removes_last_value(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(contract.sender(alice).add(bob));
        assert!(contract.sender(alice).add(charlie));
        assert!(contract.sender(alice).remove(charlie));
        assert!(!contract.sender(alice).contains(charlie));
        assert!(contract.sender(alice).contains(alice));
        assert!(contract.sender(alice).contains(bob));
        assert_eq!(contract.sender(alice).length(), uint!(2_U256));
        assert_eq!(contract.sender(alice).values(), [alice, bob]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(alice));
        assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(bob));
        assert_eq!(contract.sender(alice).at(uint!(2_U256)), None);
    }

    #[motsu::test]
    fn removes_middle_value(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(contract.sender(alice).add(bob));
        assert!(contract.sender(alice).add(charlie));
        assert!(contract.sender(alice).remove(bob));
        assert!(!contract.sender(alice).contains(bob));
        assert!(contract.sender(alice).contains(alice));
        assert!(contract.sender(alice).contains(charlie));
        assert_eq!(contract.sender(alice).length(), uint!(2_U256));
        assert_eq!(contract.sender(alice).values(), [alice, charlie]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(alice));
        assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(charlie));
        assert_eq!(contract.sender(alice).at(uint!(2_U256)), None);
    }

    #[motsu::test]
    fn does_not_remove_after_removal(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(contract.sender(alice).add(bob));
        assert!(contract.sender(alice).add(charlie));
        assert!(contract.sender(alice).remove(bob));
        assert!(!contract.sender(alice).contains(bob));
        assert!(!contract.sender(alice).remove(bob));
        assert!(contract.sender(alice).contains(alice));
        assert!(contract.sender(alice).contains(charlie));
        assert_eq!(contract.sender(alice).length(), uint!(2_U256));
        assert_eq!(contract.sender(alice).values(), [alice, charlie]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(alice));
        assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(charlie));
        assert_eq!(contract.sender(alice).at(uint!(2_U256)), None);
    }

    #[motsu::test]
    fn does_not_remove_value_not_in_set(
        contract: Contract<EnumerableAddressSet>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        assert!(contract.sender(alice).add(alice));
        assert!(contract.sender(alice).add(bob));
        assert!(!contract.sender(alice).remove(charlie));
        assert!(!contract.sender(alice).contains(charlie));
        assert!(contract.sender(alice).contains(alice));
        assert!(contract.sender(alice).contains(bob));
        assert_eq!(contract.sender(alice).length(), uint!(2_U256));
        assert_eq!(contract.sender(alice).values(), [alice, bob]);
        assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(alice));
        assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(bob));
        assert_eq!(contract.sender(alice).at(uint!(2_U256)), None);
    }
}
