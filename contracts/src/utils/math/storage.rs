//! Simple math operations missing in `stylus_sdk::storage`.
use alloy_primitives::Uint;
use alloy_sol_types::sol_data::{IntBitCount, SupportedInt};
use stylus_sdk::storage::{StorageGuardMut, StorageUint};

pub(crate) trait AddAssignUnchecked<T> {
    fn add_assign_unchecked(&mut self, rhs: T);
}

impl<'a, const B: usize, const L: usize> AddAssignUnchecked<Uint<B, L>>
    for StorageGuardMut<'a, StorageUint<B, L>>
where
    IntBitCount<B>: SupportedInt,
{
    fn add_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_value = self.get() + rhs;
        self.set(new_value);
    }
}

pub(crate) trait SubAssignUnchecked<T> {
    fn sub_assign_unchecked(&mut self, rhs: T);
}

impl<'a, const B: usize, const L: usize> SubAssignUnchecked<Uint<B, L>>
    for StorageGuardMut<'a, StorageUint<B, L>>
where
    IntBitCount<B>: SupportedInt,
{
    fn sub_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_value = self.get() - rhs;
        self.set(new_value);
    }
}
