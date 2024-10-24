//! Unit-testing context for Stylus contracts.

use std::{collections::HashMap, ptr};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use stylus_sdk::{alloy_primitives::uint, prelude::StorageType};

use crate::prelude::{Bytes32, WORD_BYTES};

/// Context of stylus unit tests associated with the current test thread.
#[allow(clippy::module_name_repetitions)]
pub struct TestContext {
    thread_name: TestThreadName,
}

impl TestContext {
    /// Get test context associated with the current test thread.
    #[must_use]
    pub fn current() -> Self {
        Self { thread_name: TestThreadName::current() }
    }

    /// Get the value at `key` in storage.
    pub(crate) fn get_bytes(self, key: &Bytes32) -> Bytes32 {
        let storage = TESTS_STORAGE.entry(self.thread_name).or_default();
        storage.contract.get(key).copied().unwrap_or_default()
    }

    /// Get the raw value at raw `key` in storage.
    pub(crate) unsafe fn get_bytes_raw(self, key: *const u8, value: *mut u8) {
        let key = read_bytes32(key);

        write_bytes32(value, self.get_bytes(&key));
    }

    /// Set the value at `key` in storage to `val`.
    pub(crate) fn set_bytes(self, key: Bytes32, val: Bytes32) {
        let mut storage = TESTS_STORAGE.entry(self.thread_name).or_default();
        storage.contract.insert(key, val);
    }

    /// Set the raw value at `key` in storage to raw `val`.
    pub(crate) unsafe fn set_bytes_raw(self, key: *const u8, value: *const u8) {
        let (key, value) = (read_bytes32(key), read_bytes32(value));
        self.set_bytes(key, value);
    }

    /// Clears storage, removing all key-value pairs associated with the current
    /// test thread.
    pub fn reset_storage(self) {
        TESTS_STORAGE.remove(&self.thread_name);
    }
}

/// Storage mock: A global mutable key-value store.
/// Allows concurrent access.
///
/// The key is the name of the test thread, and the value is the storage of the
/// test case.
static TESTS_STORAGE: Lazy<DashMap<TestThreadName, TestStorage>> =
    Lazy::new(DashMap::new);

/// Test thread name metadata.
#[derive(Clone, Eq, PartialEq, Hash)]
struct TestThreadName(String);

impl TestThreadName {
    /// Get the name of the current test thread.
    fn current() -> Self {
        let current_thread_name = std::thread::current()
            .name()
            .expect("should retrieve current thread name")
            .to_string();
        Self(current_thread_name)
    }
}

/// Context of the test case.
#[derive(Default)]
struct TestStorage {
    pub contract: HashMap<Bytes32, Bytes32>,
}

/// Read the word at address `key`.
unsafe fn read_bytes32(key: *const u8) -> Bytes32 {
    let mut res = Bytes32::default();
    ptr::copy(key, res.as_mut_ptr(), WORD_BYTES);
    res
}

/// Write the word `val` to the location pointed by `key`.
unsafe fn write_bytes32(key: *mut u8, val: Bytes32) {
    ptr::copy(val.as_ptr(), key, WORD_BYTES);
}

/// Initializes fields of contract storage and child contract storages with
/// default values.
pub trait DefaultStorage: StorageType {
    /// Initializes fields of contract storage and child contract storages with
    /// default values.
    #[must_use]
    fn default() -> Self {
        unsafe { Self::new(uint!(0_U256), 0) }
    }
}

impl<ST: StorageType> DefaultStorage for ST {}
