//! Simple math operations missing in `stylus_sdk::storage`.
use alloy_primitives::{ruint::UintTryFrom, Uint};
use stylus_sdk::storage::StorageUint;

/// Adds value and assign the result to `self`, ignoring overflow.
pub(crate) trait AddAssignUnchecked<T> {
    /// Adds `rhs` and assign the result to `self`, ignoring overflow.
    fn add_assign_unchecked(&mut self, rhs: T);
}

impl<T, const B: usize, const L: usize> AddAssignUnchecked<T>
    for StorageUint<B, L>
where
    Uint<B, L>: UintTryFrom<T>,
{
    fn add_assign_unchecked(&mut self, rhs: T) {
        let new_balance = self.get() + Uint::<B, L>::from(rhs);
        self.set(new_balance);
    }
}

/// Subtract value and assign the result to `self`, ignoring overflow.
pub(crate) trait SubAssignUnchecked<T> {
    /// Subtract `rhs` and assign the result to `self`, ignoring overflow.
    fn sub_assign_unchecked(&mut self, rhs: T);
}

impl<T, const B: usize, const L: usize> SubAssignUnchecked<T>
    for StorageUint<B, L>
where
    Uint<B, L>: UintTryFrom<T>,
{
    fn sub_assign_unchecked(&mut self, rhs: T) {
        let new_balance = self.get() - Uint::<B, L>::from(rhs);
        self.set(new_balance);
    }
}
