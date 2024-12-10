//! Simple math operations missing in `stylus_sdk::storage`.
use alloy_primitives::Uint;
use stylus_sdk::storage::{StorageGuardMut, StorageUint};

pub(crate) trait AddAssignUnchecked<T> {
    fn add_assign_unchecked(&mut self, rhs: T);
}

impl<const B: usize, const L: usize> AddAssignUnchecked<Uint<B, L>>
    for StorageGuardMut<'_, StorageUint<B, L>>
{
    fn add_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_balance = self.get() + rhs;
        self.set(new_balance);
    }
}

impl<const B: usize, const L: usize> AddAssignUnchecked<Uint<B, L>>
    for StorageUint<B, L>
{
    fn add_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_balance = self.get() + rhs;
        self.set(new_balance);
    }
}

pub(crate) trait SubAssignUnchecked<T> {
    fn sub_assign_unchecked(&mut self, rhs: T);
}

impl<const B: usize, const L: usize> SubAssignUnchecked<Uint<B, L>>
    for StorageGuardMut<'_, StorageUint<B, L>>
{
    fn sub_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_balance = self.get() - rhs;
        self.set(new_balance);
    }
}

impl<const B: usize, const L: usize> SubAssignUnchecked<Uint<B, L>>
    for StorageUint<B, L>
{
    fn sub_assign_unchecked(&mut self, rhs: Uint<B, L>) {
        let new_balance = self.get() - rhs;
        self.set(new_balance);
    }
}
