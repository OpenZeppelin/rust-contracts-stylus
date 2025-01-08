/// Module with "unchecked" math on storage values.
use alloy_primitives::Uint;
use alloy_sol_types::sol_data::{IntBitCount, SupportedInt};
use stylus_sdk::storage::StorageUint;

/// Adds value and assign the result to `self`, ignoring overflow.
pub(crate) trait AddAssignUnchecked<T> {
    /// Adds `rhs` and assign the result to `self`, ignoring overflow.
    fn add_assign_unchecked(&mut self, rhs: T);
}

impl<const B: usize, const L: usize> AddAssignUnchecked<Uint<B, L>>
    for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn add_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_balance = self.get() + rhs;
        self.set(new_balance);
    }
}

/// Subtract value and assign the result to `self`, ignoring overflow.
pub(crate) trait SubAssignUnchecked<T> {
    /// Subtract `rhs` and assign the result to `self`, ignoring overflow.
    fn sub_assign_unchecked(&mut self, rhs: T);
}

impl<const B: usize, const L: usize> SubAssignUnchecked<Uint<B, L>>
    for StorageUint<B, L>
where
    IntBitCount<B>: SupportedInt,
{
    fn sub_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_balance = self.get() - rhs;
        self.set(new_balance);
    }
}
