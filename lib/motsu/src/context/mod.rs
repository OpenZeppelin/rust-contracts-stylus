//! Unit-testing context for Stylus contracts.

use std::{collections::HashMap, ptr};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use stylus_sdk::{alloy_primitives::uint, prelude::StorageType};

use crate::prelude::{Bytes32, WORD_BYTES};

/// Context of stylus unit tests associated with the current test thread.
#[allow(clippy::module_name_repetitions)]
pub struct Context {
    thread_name: ThreadName,
}

impl Context {
    /// Get test context associated with the current test thread.
    #[must_use]
    pub fn current() -> Self {
        Self { thread_name: ThreadName::current() }
    }

    /// Get the value at `key` in storage.
    pub(crate) fn get_bytes(self, key: &Bytes32) -> Bytes32 {
        let storage = STORAGE.entry(self.thread_name).or_default();
        storage.contract_data.get(key).copied().unwrap_or_default()
    }

    /// Get the raw value at `key` in storage and write it to `value`.
    pub(crate) unsafe fn get_bytes_raw(self, key: *const u8, value: *mut u8) {
        let key = read_bytes32(key);

        write_bytes32(value, self.get_bytes(&key));
    }

    /// Set the value at `key` in storage to `value`.
    pub(crate) fn set_bytes(self, key: Bytes32, value: Bytes32) {
        let mut storage = STORAGE.entry(self.thread_name).or_default();
        storage.contract_data.insert(key, value);
    }

    /// Set the raw value at `key` in storage to `value`.
    pub(crate) unsafe fn set_bytes_raw(self, key: *const u8, value: *const u8) {
        let (key, value) = (read_bytes32(key), read_bytes32(value));
        self.set_bytes(key, value);
    }

    /// Clears storage, removing all key-value pairs associated with the current
    /// test thread.
    pub fn reset_storage(self) {
        STORAGE.remove(&self.thread_name);
    }
}

/// Storage mock: A global mutable key-value store.
/// Allows concurrent access.
///
/// The key is the name of the test thread, and the value is the storage of the
/// test case.
static STORAGE: Lazy<DashMap<ThreadName, MockStorage>> =
    Lazy::new(DashMap::new);

/// Test thread name metadata.
#[derive(Clone, Eq, PartialEq, Hash)]
struct ThreadName(String);

impl ThreadName {
    /// Get the name of the current test thread.
    fn current() -> Self {
        let current_thread_name = std::thread::current()
            .name()
            .expect("should retrieve current thread name")
            .to_string();
        Self(current_thread_name)
    }
}

/// Storage for unit test's mock data.
#[derive(Default)]
struct MockStorage {
    /// Contract's mock data storage.
    contract_data: HashMap<Bytes32, Bytes32>,
}

/// Read the word from location pointed by `ptr`.
unsafe fn read_bytes32(ptr: *const u8) -> Bytes32 {
    let mut res = Bytes32::default();
    ptr::copy(ptr, res.as_mut_ptr(), WORD_BYTES);
    res
}

/// Write the word `bytes` to the location pointed by `ptr`.
unsafe fn write_bytes32(ptr: *mut u8, bytes: Bytes32) {
    ptr::copy(bytes.as_ptr(), ptr, WORD_BYTES);
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
