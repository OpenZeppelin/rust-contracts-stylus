use alloy_primitives::U256;
use stylus_sdk::storage::{StorageGuardMut, StorageUint};

pub(crate) trait AddAssignUnchecked<T> {
    fn add_assign_unchecked(&mut self, rhs: T);
}

impl<'a> AddAssignUnchecked<U256> for StorageGuardMut<'a, StorageUint<256, 4>> {
    fn add_assign_unchecked(&mut self, rhs: U256) {
        let new_balance = self.get() + rhs;
        self.set(new_balance);
    }
}

pub(crate) trait SubAssignUnchecked<T> {
    fn sub_assign_unchecked(&mut self, rhs: T);
}

impl<'a> SubAssignUnchecked<U256> for StorageGuardMut<'a, StorageUint<256, 4>> {
    fn sub_assign_unchecked(&mut self, rhs: U256) {
        let new_balance = self.get() - rhs;
        self.set(new_balance);
    }
}
