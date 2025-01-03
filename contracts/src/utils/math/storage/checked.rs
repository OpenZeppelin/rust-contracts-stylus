//! Module with "checked" math on storage values that panics on overflow.
use alloy_primitives::Uint;
use stylus_sdk::storage::StorageUint;

/// Adds value and assign the result to `self`, panicking on overflow.
pub trait AddAssignChecked<T> {
    /// Adds `rhs` and assign the result to `self`, panicking on overflow.
    fn add_assign_checked(&mut self, rhs: T, msg: &str);
}

impl<const B: usize, const L: usize> AddAssignChecked<Uint<B, L>>
    for StorageUint<B, L>
{
    fn add_assign_checked(&mut self, rhs: Uint<B, L>, msg: &str) {
        let new_balance = self.get().checked_add(rhs).expect(msg);
        self.set(new_balance);
    }
}

/// Subtract value and assign the result to `self`, panicking on overflow.
pub trait SubAssignChecked<T> {
    /// Subtract `rhs` and assign the result to `self`, panicking on overflow.
    fn sub_assign_checked(&mut self, rhs: T, msg: &str);
}

impl<const B: usize, const L: usize> SubAssignChecked<Uint<B, L>>
    for StorageUint<B, L>
{
    fn sub_assign_checked(&mut self, rhs: Uint<B, L>, msg: &str) {
        let new_balance = self.get().checked_sub(rhs).expect(msg);
        self.set(new_balance);
    }
}
