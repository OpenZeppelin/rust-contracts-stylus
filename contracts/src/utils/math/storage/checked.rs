//! Module with "checked" math on storage values that panics on overflow.
use alloy_primitives::Uint;
use alloy_sol_types::sol_data::{IntBitCount, SupportedInt};
use stylus_sdk::storage::StorageUint;

/// Adds value and assign the result to `self`, panicking on overflow.
pub(crate) trait AddAssignChecked<T> {
    /// Adds `rhs` and assign the result to `self`, panicking on overflow.
    fn add_assign_checked(&mut self, rhs: T, msg: &str);
}

impl<const B: usize, const L: usize> AddAssignChecked<Uint<B, L>>
    for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn add_assign_checked(&mut self, rhs: Uint<B, L>, msg: &str) {
        let new_balance = self.get().checked_add(rhs).expect(msg);
        self.set(new_balance);
    }
}
