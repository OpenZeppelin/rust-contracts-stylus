//! Unit-testing utilities for Stylus contracts.
use std::sync::{Mutex, MutexGuard};

pub(crate) use wavm_shims::*;

/// A global static mutex.
///
/// We use this for scenarios where concurrent mutation of storage is wanted.
/// For example, when a test harness is running, this ensures each test
/// accesses storage in an non-overlapping manner.
///
/// See [`with_storage`].
pub(crate) static STORAGE_MUTEX: Mutex<()> = Mutex::new(());

/// Acquires access to storage.
pub(crate) fn acquire_storage() -> MutexGuard<'static, ()> {
    STORAGE_MUTEX.lock().unwrap()
}

/// Decorates a closure by running it with exclusive access to storage.
pub(crate) fn with_storage<C: Default>(closure: impl FnOnce(&mut C)) {
    let _lock = acquire_storage();
    let mut contract = C::default();
    closure(&mut contract);
    reset_storage();
}
