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

/// `EnumerableSet` trait for defining new sets.
///
/// See `impl_sets` macro
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

// Implements the [`EnumerableSet`] trait for the given types.
macro_rules! impl_set {
    ($($name:ident $skey:ident $svalue:ident)+) => {
        $(
            /// Storage for [`$name`].
            #[storage]
            pub struct $name {
                values: StorageVec<$svalue>,
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

// use alias to avoid macro issues with brackets
type Bytes32 = FixedBytes<32>;
type StorageBytes32 = StorageFixedBytes<32>;

impl_set!(
    EnumerableAddressSet Address StorageAddress
    EnumerableBytes32Set Bytes32 StorageBytes32
    EnumerableU8Set U8 StorageU8
    EnumerableU16Set U16 StorageU16
    EnumerableU32Set U32 StorageU32
    EnumerableU64Set U64 StorageU64
    EnumerableU128Set U128 StorageU128
    EnumerableU256Set U256 StorageU256
);

#[cfg(test)]
mod tests {
    use motsu::prelude::Contract;

    use super::*;

    mod address_tests {
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

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableAddressSet>,
            alice: Address,
        ) {
            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(alice));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<Address>::new());
            assert!(!contract.sender(alice).remove(alice));
        }
    }

    #[cfg(test)]
    mod bytes32_tests {

        use alloy_primitives::{uint, Address, FixedBytes};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableBytes32Set {}

        #[public]
        impl EnumerableBytes32Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableBytes32Set>,
            alice: Address,
        ) {
            let val1 = FixedBytes::<32>::repeat_byte(1);
            let val2 = FixedBytes::<32>::repeat_byte(2);
            let val3 = FixedBytes::<32>::repeat_byte(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableBytes32Set>,
            alice: Address,
        ) {
            let val = FixedBytes::<32>::repeat_byte(1);
            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(
            contract: Contract<EnumerableBytes32Set>,
            alice: Address,
        ) {
            let val1 = FixedBytes::<32>::repeat_byte(1);
            let val2 = FixedBytes::<32>::repeat_byte(2);
            let val3 = FixedBytes::<32>::repeat_byte(3);
            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableBytes32Set>,
            alice: Address,
        ) {
            let val = FixedBytes::<32>::repeat_byte(1);
            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<Bytes32>::new());
            assert!(!contract.sender(alice).remove(val));
        }
    }

    #[cfg(test)]
    mod u8_tests {
        use alloy_primitives::{uint, Address, U8};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableU8Set {}

        #[public]
        impl EnumerableU8Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableU8Set>,
            alice: Address,
        ) {
            let val1 = U8::from(1);
            let val2 = U8::from(2);
            let val3 = U8::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableU8Set>,
            alice: Address,
        ) {
            let val = U8::from(1);

            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(contract: Contract<EnumerableU8Set>, alice: Address) {
            let val1 = U8::from(1);
            let val2 = U8::from(2);
            let val3 = U8::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableU8Set>,
            alice: Address,
        ) {
            let val = U8::from(1);

            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<U8>::new());
            assert!(!contract.sender(alice).remove(val));
        }

        #[motsu::test]
        fn boundary_values(
            contract: Contract<EnumerableU8Set>,
            alice: Address,
        ) {
            let min_val = U8::ZERO;
            let max_val = U8::MAX;

            assert!(contract.sender(alice).add(min_val));
            assert!(contract.sender(alice).add(max_val));
            assert!(contract.sender(alice).contains(min_val));
            assert!(contract.sender(alice).contains(max_val));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [min_val, max_val]);
        }
    }

    #[cfg(test)]
    mod u16_tests {
        use alloy_primitives::{uint, Address, U16};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableU16Set {}

        #[public]
        impl EnumerableU16Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableU16Set>,
            alice: Address,
        ) {
            let val1 = U16::from(1);
            let val2 = U16::from(2);
            let val3 = U16::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableU16Set>,
            alice: Address,
        ) {
            let val = U16::from(1);

            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(
            contract: Contract<EnumerableU16Set>,
            alice: Address,
        ) {
            let val1 = U16::from(1);
            let val2 = U16::from(2);
            let val3 = U16::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableU16Set>,
            alice: Address,
        ) {
            let val = U16::from(1);

            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<U16>::new());
            assert!(!contract.sender(alice).remove(val));
        }

        #[motsu::test]
        fn boundary_values(
            contract: Contract<EnumerableU16Set>,
            alice: Address,
        ) {
            let min_val = U16::ZERO;
            let max_val = U16::MAX;

            assert!(contract.sender(alice).add(min_val));
            assert!(contract.sender(alice).add(max_val));
            assert!(contract.sender(alice).contains(min_val));
            assert!(contract.sender(alice).contains(max_val));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [min_val, max_val]);
        }
    }

    #[cfg(test)]
    mod u32_tests {
        use alloy_primitives::{uint, Address, U32};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableU32Set {}

        #[public]
        impl EnumerableU32Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableU32Set>,
            alice: Address,
        ) {
            let val1 = U32::from(1);
            let val2 = U32::from(2);
            let val3 = U32::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableU32Set>,
            alice: Address,
        ) {
            let val = U32::from(1);

            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(
            contract: Contract<EnumerableU32Set>,
            alice: Address,
        ) {
            let val1 = U32::from(1);
            let val2 = U32::from(2);
            let val3 = U32::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableU32Set>,
            alice: Address,
        ) {
            let val = U32::from(1);

            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<U32>::new());
            assert!(!contract.sender(alice).remove(val));
        }

        #[motsu::test]
        fn boundary_values(
            contract: Contract<EnumerableU32Set>,
            alice: Address,
        ) {
            let min_val = U32::ZERO;
            let max_val = U32::MAX;

            assert!(contract.sender(alice).add(min_val));
            assert!(contract.sender(alice).add(max_val));
            assert!(contract.sender(alice).contains(min_val));
            assert!(contract.sender(alice).contains(max_val));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [min_val, max_val]);
        }
    }

    #[cfg(test)]
    mod u64_tests {
        use alloy_primitives::{uint, Address, U64};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableU64Set {}

        #[public]
        impl EnumerableU64Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableU64Set>,
            alice: Address,
        ) {
            let val1 = U64::from(1);
            let val2 = U64::from(2);
            let val3 = U64::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableU64Set>,
            alice: Address,
        ) {
            let val = U64::from(1);

            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(
            contract: Contract<EnumerableU64Set>,
            alice: Address,
        ) {
            let val1 = U64::from(1);
            let val2 = U64::from(2);
            let val3 = U64::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableU64Set>,
            alice: Address,
        ) {
            let val = U64::from(1);

            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<U64>::new());
            assert!(!contract.sender(alice).remove(val));
        }

        #[motsu::test]
        fn boundary_values(
            contract: Contract<EnumerableU64Set>,
            alice: Address,
        ) {
            let min_val = U64::ZERO;
            let max_val = U64::MAX;

            assert!(contract.sender(alice).add(min_val));
            assert!(contract.sender(alice).add(max_val));
            assert!(contract.sender(alice).contains(min_val));
            assert!(contract.sender(alice).contains(max_val));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [min_val, max_val]);
        }
    }

    #[cfg(test)]
    mod u128_tests {
        use alloy_primitives::{uint, Address, U128};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableU128Set {}

        #[public]
        impl EnumerableU128Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableU128Set>,
            alice: Address,
        ) {
            let val1 = U128::from(1);
            let val2 = U128::from(2);
            let val3 = U128::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableU128Set>,
            alice: Address,
        ) {
            let val = U128::from(1);

            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(
            contract: Contract<EnumerableU128Set>,
            alice: Address,
        ) {
            let val1 = U128::from(1);
            let val2 = U128::from(2);
            let val3 = U128::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableU128Set>,
            alice: Address,
        ) {
            let val = U128::from(1);

            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<U128>::new());
            assert!(!contract.sender(alice).remove(val));
        }

        #[motsu::test]
        fn boundary_values(
            contract: Contract<EnumerableU128Set>,
            alice: Address,
        ) {
            let min_val = U128::ZERO;
            let max_val = U128::MAX;

            assert!(contract.sender(alice).add(min_val));
            assert!(contract.sender(alice).add(max_val));
            assert!(contract.sender(alice).contains(min_val));
            assert!(contract.sender(alice).contains(max_val));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [min_val, max_val]);
        }
    }

    #[cfg(test)]
    mod u256_tests {
        use alloy_primitives::{uint, Address, U256};
        use motsu::prelude::Contract;
        use stylus_sdk::prelude::{public, TopLevelStorage};

        use super::*;

        unsafe impl TopLevelStorage for EnumerableU256Set {}

        #[public]
        impl EnumerableU256Set {}

        #[motsu::test]
        fn add_multiple_values(
            contract: Contract<EnumerableU256Set>,
            alice: Address,
        ) {
            let val1 = U256::from(1);
            let val2 = U256::from(2);
            let val3 = U256::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(3_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val2, val3]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val1));
            assert_eq!(contract.sender(alice).at(uint!(1_U256)), Some(val2));
            assert_eq!(contract.sender(alice).at(uint!(2_U256)), Some(val3));
            assert_eq!(contract.sender(alice).at(uint!(3_U256)), None);
        }

        #[motsu::test]
        fn does_not_duplicate_values(
            contract: Contract<EnumerableU256Set>,
            alice: Address,
        ) {
            let val = U256::from(1);

            assert!(contract.sender(alice).add(val));
            assert!(!contract.sender(alice).add(val));
            assert!(contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).length(), uint!(1_U256));
            assert_eq!(contract.sender(alice).values(), [val]);
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), Some(val));
        }

        #[motsu::test]
        fn removes_values(
            contract: Contract<EnumerableU256Set>,
            alice: Address,
        ) {
            let val1 = U256::from(1);
            let val2 = U256::from(2);
            let val3 = U256::from(3);

            assert!(contract.sender(alice).add(val1));
            assert!(contract.sender(alice).add(val2));
            assert!(contract.sender(alice).add(val3));
            assert!(contract.sender(alice).remove(val2));
            assert!(!contract.sender(alice).contains(val2));
            assert!(contract.sender(alice).contains(val1));
            assert!(contract.sender(alice).contains(val3));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [val1, val3]);
        }

        #[motsu::test]
        fn empty_set_operations(
            contract: Contract<EnumerableU256Set>,
            alice: Address,
        ) {
            let val = U256::from(1);

            assert_eq!(contract.sender(alice).length(), uint!(0_U256));
            assert!(!contract.sender(alice).contains(val));
            assert_eq!(contract.sender(alice).at(uint!(0_U256)), None);
            assert_eq!(contract.sender(alice).values(), Vec::<U256>::new());
            assert!(!contract.sender(alice).remove(val));
        }

        #[motsu::test]
        fn boundary_values(
            contract: Contract<EnumerableU256Set>,
            alice: Address,
        ) {
            let min_val = U256::ZERO;
            let max_val = U256::MAX;

            assert!(contract.sender(alice).add(min_val));
            assert!(contract.sender(alice).add(max_val));
            assert!(contract.sender(alice).contains(min_val));
            assert!(contract.sender(alice).contains(max_val));
            assert_eq!(contract.sender(alice).length(), uint!(2_U256));
            assert_eq!(contract.sender(alice).values(), [min_val, max_val]);
        }
    }
}
