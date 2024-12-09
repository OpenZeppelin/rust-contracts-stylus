//! Unit-testing context for Stylus contracts.

use std::{collections::HashMap, ptr};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use stylus_sdk::{alloy_primitives::uint, prelude::StorageType};

use crate::prelude::{Bytes32, WORD_BYTES};

mod environment;

use environment::Environment;

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
        let context = EVM.entry(self.thread_name).or_default();
        context.storage.contract_data.get(key).copied().unwrap_or_default()
    }

    /// Get the raw value at `key` in storage and write it to `value`.
    pub(crate) unsafe fn get_bytes_raw(self, key: *const u8, value: *mut u8) {
        let key = read_bytes32(key);

        write_bytes32(value, self.get_bytes(&key));
    }

    /// Set the value at `key` in storage to `value`.
    pub(crate) fn set_bytes(self, key: Bytes32, value: Bytes32) {
        let mut context = EVM.entry(self.thread_name).or_default();
        context.storage.contract_data.insert(key, value);
    }

    /// Set the raw value at `key` in storage to `value`.
    pub(crate) unsafe fn set_bytes_raw(self, key: *const u8, value: *const u8) {
        let (key, value) = (read_bytes32(key), read_bytes32(value));
        self.set_bytes(key, value);
    }

    /// Clears storage, removing all key-value pairs associated with the current
    /// test thread.
    pub fn reset_storage(self) {
        EVM.remove(&self.thread_name);
    }

    /// Gets the code hash of the account at the given address.
    pub fn account_codehash(self) -> [u8; 66] {
        let context = EVM.entry(self.thread_name).or_default();
        context.environment.account_codehash()
    }

    /// Gets a bounded estimate of the Unix timestamp at which the Sequencer
    /// sequenced the transaction.
    pub fn block_timestamp(self) -> u64 {
        let context = EVM.entry(self.thread_name).or_default();
        context.environment.block_timestamp()
    }

    /// Gets the chain ID of the current chain.
    pub fn chain_id(self) -> u64 {
        let context = EVM.entry(self.thread_name).or_default();
        context.environment.chain_id()
    }

    /// Gets the address of the current program.
    pub fn contract_address(self) -> [u8; 42] {
        let context = EVM.entry(self.thread_name).or_default();
        context.environment.contract_address()
    }

    /// Emits an EVM log with the given number of topics and data, the first
    /// bytes of which should be the 32-byte-aligned topic data.
    ///
    /// Data contains `topics` amount of topics and then encoded event in `data`
    /// buffer.
    pub(crate) unsafe fn emit_log(
        self,
        data: *const u8,
        len: usize,
        topics: usize,
    ) {
        // https://github.com/OffchainLabs/stylus-sdk-rs/blob/v0.6.0/stylus-sdk/src/evm.rs#L38-L52
        let buffer = read_bytes(data, len);
        let encoded_event: Vec<u8> =
            buffer.into_iter().skip(topics * WORD_BYTES).collect();
        let mut context = EVM.entry(self.thread_name).or_default();
        context.environment.store_event(&encoded_event);
    }

    /// Gets the address of the account that called the program.
    pub fn msg_sender(self) -> [u8; 42] {
        let context = EVM.entry(self.thread_name).or_default();
        context.environment.msg_sender()
    }

    /// Removes all events for a test case.
    pub fn clear_events(self) {
        let mut context = EVM.entry(self.thread_name).or_default();
        context.environment.clear_events();
    }

    /// Gets all emitted events for a test case.
    pub fn events(self) -> Vec<Vec<u8>> {
        let context = EVM.entry(self.thread_name).or_default();
        context.environment.events()
    }
}

#[derive(Default)]
struct TestCase {
    storage: MockStorage,
    environment: Environment,
}

/// A global mutable key-value store mockig EVM behaviour.
/// Allows concurrent access.
///
/// The key is the name of the test thread,
/// and the value is the context of the test case.
static EVM: Lazy<DashMap<ThreadName, TestCase>> = Lazy::new(DashMap::new);

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

/// Read data from location pointed by `ptr`.
unsafe fn read_bytes(ptr: *const u8, len: usize) -> Vec<u8> {
    let mut res = Vec::with_capacity(len);
    ptr::copy(ptr, res.as_mut_ptr(), len);
    res
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
