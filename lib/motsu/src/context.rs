//! Unit-testing context for Stylus contracts.

use std::{borrow::BorrowMut, collections::HashMap, ptr, slice, sync::Mutex};

use alloy_primitives::Address;
use dashmap::{mapref::one::RefMut, DashMap};
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
        Self { thread_name: ThreadName::current() }
    }

    /// Get the raw value at `key` in storage and write it to `value`.
    pub(crate) unsafe fn get_bytes_raw(self, key: *const u8, value: *mut u8) {
        let key = read_bytes32(key);

        write_bytes32(value, self.get_bytes(&key));
    }

    /// Get the value at `key` in storage.
    fn get_bytes(self, key: &Bytes32) -> Bytes32 {
        let storage = self.get_storage();
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

    /// Set the raw value at `key` in storage to `value`.
    pub(crate) unsafe fn set_bytes_raw(self, key: *const u8, value: *const u8) {
        let (key, value) = (read_bytes32(key), read_bytes32(value));
        self.set_bytes(key, value);
    }

    /// Set the value at `key` in storage to `value`.
    fn set_bytes(self, key: Bytes32, value: Bytes32) {
        let mut storage = self.get_storage();
        let msg_receiver =
            storage.msg_receiver.expect("msg_receiver should be set");
        storage
            .contract_data
            .get_mut(&msg_receiver)
            .expect("contract receiver should have a storage initialised")
            .insert(key, value);
    }

    /// Clears storage, removing all key-value pairs associated with the current
    /// test thread.
    pub fn reset_storage(self) {
        STORAGE.remove(&self.thread_name);
    }

    /// Set the message sender account address.
    fn set_msg_sender(&self, msg_sender: Address) -> Option<Address> {
        self.get_storage().msg_sender.replace(msg_sender)
    }

    /// Get the message sender account address.
    pub fn get_msg_sender(&self) -> Option<Address> {
        self.get_storage().msg_sender
    }

    /// Set the address of the contract, that should be called.
    fn set_msg_receiver(&self, msg_receiver: Address) -> Option<Address> {
        self.get_storage().msg_receiver.replace(msg_receiver)
    }

    /// Get the address of the contract, that should be called.
    fn get_msg_receiver(&self) -> Option<Address> {
        self.get_storage().msg_receiver
    }

    /// Initialise contract storage for the current test thread and
    /// `contract_address`.
    fn init_contract<ST: StorageType + TestRouter + 'static>(
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
            .entry(self.thread_name)
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

    /// Call the contract at raw `address` with the given raw `calldata`.
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
                return_data_len.write(err.len());
                self.set_return_data(err);
                1
            }
        }
    }

    fn call_contract(
        &self,
        contract_address: Address,
        selector: u32,
        input: &[u8],
    ) -> ArbResult {
        // Set the current contract as message sender and callee contract as
        // receiver.
        let previous_receiver = self
            .set_msg_receiver(contract_address)
            .expect("msg_receiver should be set");
        let previous_msg_sender = self
            .set_msg_sender(previous_receiver)
            .expect("msg_sender should be set");

        // Call external contract.
        let call_storage = self.get_call_storage();
        let router = call_storage
            .contract_router
            .get(&contract_address)
            .expect("contract router should be set");
        let mut router = router.lock().expect("should lock test router");
        let result = router.route(selector, input).unwrap_or_else(|| {
            panic!("selector not found - selector: {selector}")
        });

        // Set the previous message sender and receiver back.
        let _ = self.set_msg_receiver(previous_receiver);
        let _ = self.set_msg_sender(previous_msg_sender);

        result
    }

    fn set_return_data(&self, data: Vec<u8>) {
        let mut call_storage = self.get_call_storage();
        let _ = call_storage.call_output_len.insert(data.len());
        let _ = call_storage.call_output.insert(data);
    }

    pub(crate) unsafe fn read_return_data_raw(
        self,
        dest: *mut u8,
        size: usize,
    ) -> usize {
        let data = self.get_return_data();
        ptr::copy(data.as_ptr(), dest, size);
        data.len()
    }

    pub(crate) fn get_return_data_size(&self) -> usize {
        self.get_call_storage()
            .call_output_len
            .take()
            .expect("call_output_len should be set")
    }

    fn get_return_data(&self) -> Vec<u8> {
        self.get_call_storage()
            .call_output
            .take()
            .expect("call_output should be set")
    }

    /// Check if the contract at raw `address` has code.
    pub(crate) unsafe fn has_code_raw(self, address: *const u8) -> bool {
        let address_bytes = slice::from_raw_parts(address, 20);
        let address = Address::from_slice(address_bytes);
        self.has_code(address)
    }

    /// Check if the contract at `address` has code.
    #[must_use]
    fn has_code(&self, address: Address) -> bool {
        let call_storage = self.get_call_storage();
        call_storage.contract_router.contains_key(&address)
    }

    /// Get reference to the storage for the current test thread.
    fn get_storage(&self) -> RefMut<'static, ThreadName, MockStorage> {
        STORAGE
            .get_mut(&self.thread_name)
            .expect("contract should be initialised first")
    }

    /// Get reference to the call storage for the current test thread.
    fn get_call_storage(&self) -> RefMut<'static, ThreadName, CallStorage> {
        CALL_STORAGE
            .get_mut(&self.thread_name.clone())
            .expect("contract should be initialised first")
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

/// Storage for unit test's mock data.
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
    contract_router: HashMap<Address, Mutex<Box<dyn TestRouter>>>,
    // Output of a contract call.
    call_output: Option<Vec<u8>>,
    // Output length of a contract call.
    call_output_len: Option<usize>,
}

/// A trait for routing messages to the appropriate selector in tests.
pub trait TestRouter: Send {
    /// Tries to find and execute a method for the given selector, returning
    /// `None` if none is found.
    fn route(&mut self, selector: u32, input: &[u8]) -> Option<ArbResult>;
}

impl<R> TestRouter for R
where
    R: Router<R> + TopLevelStorage + BorrowMut<R::Storage> + Send,
{
    fn route(&mut self, selector: u32, input: &[u8]) -> Option<ArbResult> {
        <Self as Router<R>>::route(self, selector, input)
    }
}

impl<ST: StorageType + TestRouter + 'static> Default for Contract<ST> {
    fn default() -> Self {
        Contract::random()
    }
}

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
        let _ = Context::current().set_msg_sender(self.caller_address);
        let _ = Context::current().set_msg_receiver(self.contract_address);
        &self.contract
    }
}

impl<ST: StorageType> ::core::ops::DerefMut for ContractCall<ST> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let _ = Context::current().set_msg_sender(self.caller_address);
        let _ = Context::current().set_msg_receiver(self.contract_address);
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

    pub fn sender(&self, account: Account) -> ContractCall<ST> {
        ContractCall {
            contract: unsafe { ST::new(uint!(0_U256), 0) },
            caller_address: account.address,
            contract_address: self.address,
        }
    }
}

#[derive(Clone, Copy)]
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
}
