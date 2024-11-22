//! Unit-testing context for Stylus contracts.

use std::{borrow::BorrowMut, collections::HashMap, ptr, slice, sync::Mutex};

use alloy_primitives::Address;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use stylus_sdk::{
    abi::Router,
    alloy_primitives::uint,
    prelude::{StorageType, TopLevelStorage},
    ArbResult,
};

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
        // TODO#q: STORAGE entry call here
        Self { thread_name: ThreadName::current() }
    }

    /// Get the value at `key` in storage.
    pub(crate) fn get_bytes(self, key: &Bytes32) -> Bytes32 {
        // TODO#q: fix deadlock here.
        //  When contract is called from another contract, it access storage
        // second time.  Split STORAGE into two parts.
        let storage = STORAGE.entry(self.thread_name).or_default();
        let msg_receiver =
            storage.msg_receiver.expect("msg_receiver should be set");
        storage
            .contract_data
            .get(&msg_receiver)
            .expect("contract receiver should have a storage initialised")
            .get(key)
            .copied()
            .unwrap_or_default()
    }

    /// Get the raw value at `key` in storage and write it to `value`.
    pub(crate) unsafe fn get_bytes_raw(self, key: *const u8, value: *mut u8) {
        let key = read_bytes32(key);

        write_bytes32(value, self.get_bytes(&key));
    }

    /// Set the value at `key` in storage to `value`.
    pub(crate) fn set_bytes(self, key: Bytes32, value: Bytes32) {
        let mut storage = STORAGE.entry(self.thread_name).or_default();
        let msg_receiver =
            storage.msg_receiver.expect("msg_receiver should be set");
        storage
            .contract_data
            .get_mut(&msg_receiver)
            .expect("contract receiver should have a storage initialised")
            .insert(key, value);
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

    pub(crate) fn set_msg_sender(self, msg_sender: Address) {
        let mut storage = STORAGE.entry(self.thread_name).or_default();
        let _ = storage.msg_sender.insert(msg_sender);
    }

    pub(crate) fn get_msg_sender(self) -> Address {
        let storage = STORAGE.entry(self.thread_name).or_default();
        storage.msg_sender.expect("msg_sender should be set")
    }

    pub(crate) fn set_msg_receiver(self, msg_receiver: Address) {
        let mut storage = STORAGE.entry(self.thread_name).or_default();
        let _ = storage.msg_receiver.insert(msg_receiver);
    }

    pub(crate) fn get_msg_receiver(self) -> Address {
        let storage = STORAGE.entry(self.thread_name).or_default();
        storage.msg_receiver.expect("msg_receiver should be set")
    }

    pub(crate) fn init_contract<ST: StorageType + TestRouter + 'static>(
        self,
        contract_address: Address,
    ) {
        if STORAGE
            .entry(self.thread_name.clone())
            .or_default()
            .contract_data
            .insert(contract_address, HashMap::new())
            .is_some()
        {
            panic!("contract storage already initialized - contract_address: {contract_address}");
        }

        if CALL_STORAGE
            .entry(self.thread_name.clone())
            .or_default()
            .contract_router
            .insert(
                contract_address,
                Mutex::new(Box::new(unsafe { ST::new(uint!(0_U256), 0) })),
            )
            .is_some()
        {
            panic!("contract storage already initialized - contract_address: {contract_address}");
        }
    }

    pub(crate) unsafe fn call_contract_raw(
        self,
        address: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        return_data_len: *mut usize,
    ) -> u8 {
        let address_bytes = slice::from_raw_parts(address, 20);
        let address = Address::from_slice(address_bytes);

        let input = slice::from_raw_parts(calldata, calldata_len);
        let selector =
            u32::from_be_bytes(TryInto::try_into(&input[..4]).unwrap());

        match self.call_contract(address, selector, &input[4..]) {
            Ok(res) => {
                return_data_len.write(res.len());
                self.set_return_data(res);
                0
            }
            Err(err) => {
                // TODO#q: how should we process errors?
                return_data_len.write(err.len());
                self.set_return_data(err);
                1
            }
        }
    }

    pub(crate) fn set_return_data(&self, data: Vec<u8>) {
        let _ = CALL_STORAGE
            .entry(self.thread_name.clone())
            .or_default()
            .call_output
            .insert(data);
    }

    pub(crate) fn call_contract(
        &self,
        contract_address: Address,
        selector: u32,
        input: &[u8],
    ) -> ArbResult {
        let mut storage = STORAGE.entry(self.thread_name.clone()).or_default();
        let previous_receiver = storage.msg_receiver.replace(contract_address);
        let previous_sender = storage.msg_sender.take();
        storage.msg_sender = previous_receiver; // now the sender is current contract
        drop(storage);

        let call_storage =
            CALL_STORAGE.entry(self.thread_name.clone()).or_default();
        let router = call_storage
            .contract_router
            .get(&contract_address)
            .expect("contract router should be set");
        let mut router = router.lock().expect("should lock test router");
        let result = router.route(selector, input).unwrap_or_else(|| {
            panic!("selector not found - selector: {selector}")
        });

        let mut storage = STORAGE.entry(self.thread_name.clone()).or_default();
        storage.msg_receiver = previous_receiver;
        storage.msg_sender = previous_sender;

        result
    }

    pub(crate) unsafe fn read_return_data_raw(
        self,
        dest: *mut u8,
        size: usize,
    ) -> usize {
        let data = self.get_return_data();
        ptr::copy(data.as_ptr(), dest, size);
        0
    }

    pub(crate) fn get_return_data(&self) -> Vec<u8> {
        CALL_STORAGE
            .entry(self.thread_name.clone())
            .or_default()
            .call_output
            .take()
            .expect("call_output should be set")
    }
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

/// Storage for unit test's mock data.x
#[derive(Default)]
struct MockStorage {
    /// Address of the message sender.
    msg_sender: Option<Address>,
    /// Address of the contract that is currently receiving the message.
    msg_receiver: Option<Address>,
    /// Contract's address to mock data storage mapping.
    contract_data: HashMap<Address, ContractStorage>,
}

type ContractStorage = HashMap<Bytes32, Bytes32>;

/// The key is the name of the test thread, and the value is external call
/// metadata.
static CALL_STORAGE: Lazy<DashMap<ThreadName, CallStorage>> =
    Lazy::new(DashMap::new);

/// Metadata related to call of external contract.
#[derive(Default)]
struct CallStorage {
    // Contract's address to router mapping.
    // NOTE: Mutex is important since contract type is not `Sync`.
    contract_router: HashMap<Address, std::sync::Mutex<Box<dyn TestRouter>>>,
    // Output of a contract call.
    call_output: Option<Vec<u8>>,
}

/// A trait for routing messages to the appropriate selector in tests.
pub trait TestRouter: Send {
    /// Tries to find and execute a method for the given selector, returning
    /// `None` if none is found.
    fn route(&mut self, selector: u32, input: &[u8]) -> Option<ArbResult>;
}

impl<R: Router<R> + TopLevelStorage + BorrowMut<R::Storage> + Send> TestRouter
    for R
{
    fn route(&mut self, selector: u32, input: &[u8]) -> Option<ArbResult> {
        <Self as Router<R>>::route(self, selector, input)
    }
}

/// Initializes fields of contract storage and child contract storages with
/// default values.
pub trait DefaultStorage: StorageType + TestRouter + 'static {
    /// Initializes fields of contract storage and child contract storages with
    /// default values.
    #[must_use]
    fn default() -> Contract<Self> {
        Contract::random()
    }
}

impl<ST: StorageType + TestRouter + 'static> DefaultStorage for ST {}

pub struct ContractCall<ST: StorageType> {
    contract: ST,
    caller_address: Address,
    contract_address: Address,
}

impl<ST: StorageType> ContractCall<ST> {
    pub fn address(&self) -> Address {
        self.contract_address
    }
}

impl<ST: StorageType> ::core::ops::Deref for ContractCall<ST> {
    type Target = ST;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Context::current().set_msg_sender(self.caller_address);
        Context::current().set_msg_receiver(self.contract_address);
        &self.contract
    }
}

impl<ST: StorageType> ::core::ops::DerefMut for ContractCall<ST> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Context::current().set_msg_sender(self.caller_address);
        Context::current().set_msg_receiver(self.contract_address);
        &mut self.contract
    }
}

pub struct Contract<ST: StorageType> {
    phantom: ::core::marker::PhantomData<ST>,
    address: Address,
}

impl<ST: StorageType + TestRouter + 'static> Contract<ST> {
    pub fn new(address: Address) -> Self {
        Context::current().init_contract::<ST>(address);

        Self { phantom: ::core::marker::PhantomData, address }
    }

    // TODO#q: probably we need generic initializer

    pub fn random() -> Self {
        Self::new(Address::random())
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

pub struct Account {
    address: Address,
}

impl Account {
    #[must_use]
    pub const fn new(address: Address) -> Self {
        Self { address }
    }

    #[must_use]
    pub fn random() -> Self {
        Self::new(Address::random())
    }

    #[must_use]
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn uses<ST: StorageType>(
        &self,
        contract: &mut Contract<ST>,
    ) -> ContractCall<ST> {
        ContractCall {
            contract: unsafe { ST::new(uint!(0_U256), 0) },
            caller_address: self.address,
            contract_address: contract.address,
        }
    }
}
