//! The simulated chain shared by every contract in a test.
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Debug},
    rc::Rc,
};

use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::SolEvent;
use stylus_sdk::{
    abi::router_entrypoint, host::VM, prelude::*, storage::StorageType,
    testing::TestVM, ArbResult,
};

use crate::{contract::Contract, host::EnvHost, Routable};

/// Bytecode reported for every deployed contract.
///
/// Stylus programs start with the EOF prefix; only the length matters here.
pub const CONTRACT_CODE: &[u8] = &[0xEF, 0xF0, 0x00, 0x00];

thread_local! {
    static CURRENT: RefCell<Option<Env>> = const { RefCell::new(None) };
}

/// A call dispatcher: routes raw calldata to a contract type's router.
type Dispatch = Rc<dyn Fn(Vec<u8>, VM) -> ArbResult>;

/// How a call reaches a contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CallKind {
    /// A regular call: new storage context, may transfer value.
    Call,
    /// A static call: new storage context, writes are rejected.
    StaticCall,
    /// A delegate call: the caller's storage context and message are kept.
    DelegateCall,
}

/// A log emitted during the test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Log {
    /// Address of the contract that emitted the log.
    pub emitter: Address,
    /// Log topics, including the event signature.
    pub topics: Vec<B256>,
    /// ABI-encoded non-indexed data.
    pub data: Vec<u8>,
}

impl Log {
    /// Decodes the log as the event `E`, if it matches.
    #[must_use]
    pub fn decode<E: SolEvent>(&self) -> Option<E> {
        E::decode_raw_log(self.topics.iter().copied(), &self.data).ok()
    }

    /// Returns `true` if this log is the ABI encoding of `event`.
    #[must_use]
    pub fn matches<E: SolEvent>(&self, event: &E) -> bool {
        let expected = event.encode_log_data();
        expected.topics() == self.topics.as_slice()
            && expected.data.as_ref() == self.data.as_slice()
    }
}

/// One active call on the simulated call stack.
#[derive(Clone, Debug)]
pub(crate) struct Frame {
    /// `msg.sender` seen by the executing code.
    pub(crate) sender: Address,
    /// `address(this)`: the storage context of the executing code.
    pub(crate) this: Address,
    /// `msg.value`.
    pub(crate) value: U256,
    /// Whether storage writes are rejected.
    pub(crate) is_static: bool,
    /// Set when a static frame attempted a storage write.
    pub(crate) static_violation: bool,
}

/// A contract registered in the environment.
pub(crate) struct Deployed {
    /// Holds the contract's storage and the mocks for its external calls.
    pub(crate) vm: TestVM,
    /// Routes calls to the contract's `#[public]` methods, if it has any.
    dispatch: Option<Dispatch>,
}

/// Snapshot of everything a revert must roll back.
struct Checkpoint {
    storage: HashMap<Address, HashMap<U256, B256>>,
    balances: HashMap<Address, U256>,
    logs: usize,
}

pub(crate) struct State {
    /// Balances, code and the block, chain and transaction context.
    pub(crate) chain: TestVM,
    pub(crate) contracts: HashMap<Address, Deployed>,
    pub(crate) frames: Vec<Frame>,
    pub(crate) logs: Vec<Log>,
    pub(crate) return_data: Vec<u8>,
    names: HashMap<Address, String>,
    /// Checkpoint taken when the last top-level call started.
    pending: Option<Checkpoint>,
    /// Sender and target of the last top-level call, for messages.
    last_call: Option<(Address, Address)>,
}

impl State {
    /// The frame whose code is executing.
    ///
    /// # Panics
    ///
    /// If no call is in progress.
    pub(crate) fn frame(&self) -> &Frame {
        self.frames.last().expect(
            "contracts can only be used through `Contract::sender` or a \
             cross-contract call",
        )
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            storage: self
                .contracts
                .iter()
                .map(|(address, deployed)| {
                    (*address, deployed.vm.snapshot().storage)
                })
                .collect(),
            balances: self.chain.snapshot().balances,
            logs: self.logs.len(),
        }
    }

    fn restore(&mut self, checkpoint: Checkpoint) {
        for (address, storage) in checkpoint.storage {
            if let Some(deployed) = self.contracts.get(&address) {
                deployed.vm.clear_storage();
                for (slot, value) in storage {
                    deployed.vm.set_storage(slot, value);
                }
            }
        }
        let current = self.chain.snapshot().balances;
        for address in current.keys().chain(checkpoint.balances.keys()) {
            let balance =
                checkpoint.balances.get(address).copied().unwrap_or_default();
            self.chain.set_balance(*address, balance);
        }
        self.logs.truncate(checkpoint.logs);
    }

    fn transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), ()> {
        if value.is_zero() {
            return Ok(());
        }
        let Some(new_from_balance) = self.chain.balance(from).checked_sub(value)
        else {
            return Err(());
        };
        let new_to_balance = self
            .chain
            .balance(to)
            .checked_add(value)
            .expect("balance should not overflow");
        self.chain.set_balance(from, new_from_balance);
        self.chain.set_balance(to, new_to_balance);
        Ok(())
    }
}

/// A simulated chain holding any number of contracts and accounts.
///
/// Cloning an [`Env`] gives another handle to the same state. The most
/// recently created environment of the thread is the *current* one, used by
/// [`ResultExt`](crate::ResultExt), [`Funding`](crate::Funding) and
/// [`Balance`](crate::Balance).
#[derive(Clone)]
pub struct Env {
    pub(crate) state: Rc<RefCell<State>>,
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.borrow();
        f.debug_struct("Env")
            .field("contracts", &state.contracts.len())
            .field("block_number", &state.chain.block_number())
            .field("block_timestamp", &state.chain.block_timestamp())
            .field("chain_id", &state.chain.chain_id())
            .finish_non_exhaustive()
    }
}

impl Env {
    /// Creates an empty environment and makes it the current one.
    #[must_use]
    pub fn new() -> Self {
        let env = Self {
            state: Rc::new(RefCell::new(State {
                chain: TestVM::new(),
                contracts: HashMap::new(),
                frames: Vec::new(),
                logs: Vec::new(),
                return_data: Vec::new(),
                names: HashMap::new(),
                pending: None,
                last_call: None,
            })),
        };
        CURRENT.with(|current| *current.borrow_mut() = Some(env.clone()));
        env
    }

    /// The current environment of this thread.
    ///
    /// # Panics
    ///
    /// If no environment was created on this thread.
    #[must_use]
    pub fn current() -> Self {
        CURRENT.with(|current| current.borrow().clone()).expect(
            "no test environment: create one with `Env::new()` or \
             `#[testing::test]`",
        )
    }

    /// The host that contracts use to reach this environment.
    #[must_use]
    pub fn host(&self) -> EnvHost {
        EnvHost { env: self.clone() }
    }

    /// The SDK VM wrapper around [`Env::host`].
    #[must_use]
    pub fn vm(&self) -> VM {
        VM { host: Box::new(self.host()) }
    }

    /// The SDK test VM holding the balances, code and block, chain and
    /// transaction context shared by every contract.
    ///
    /// Use its setters to change the block number, timestamp, chain id or
    /// `tx.origin`.
    #[must_use]
    pub fn chain(&self) -> TestVM {
        self.state.borrow().chain.clone()
    }

    /// Creates an account with no balance.
    ///
    /// The address is derived from `name`, so the same name always gives the
    /// same address.
    pub fn account(&self, name: &str) -> Address {
        let address = derive_address("account", name);
        self.state.borrow_mut().names.insert(address, name.to_owned());
        address
    }

    /// Records `name` as the test name of `address`, for messages.
    pub fn name_address(&self, address: Address, name: &str) {
        self.state.borrow_mut().names.insert(address, name.to_owned());
    }

    /// Deploys the contract type `T` at an address derived from `name`.
    ///
    /// Calls from other contracts are routed to `T`'s `#[public]` methods.
    /// The `#[constructor]`, if any, is not run: call it explicitly.
    pub fn deploy<T: Routable>(&self, name: &str) -> Contract<T> {
        let dispatch: Dispatch =
            Rc::new(|input, vm| router_entrypoint::<T, T>(input, vm));
        self.register(name, Some(dispatch))
    }

    /// Creates a handle to the storage type `T` at an address derived from
    /// `name` without routing calls to it.
    ///
    /// Use it for `#[storage]` types that have no `#[public]` inherent impl
    /// block. Other contracts calling this address see no code.
    pub fn attach<T: StorageType>(&self, name: &str) -> Contract<T> {
        self.register(name, None)
    }

    fn register<T: StorageType>(
        &self,
        name: &str,
        dispatch: Option<Dispatch>,
    ) -> Contract<T> {
        let address = derive_address("contract", name);
        let mut state = self.state.borrow_mut();
        if dispatch.is_some() {
            state.chain.set_code(address, CONTRACT_CODE.to_vec());
        }
        state.names.insert(address, name.to_owned());
        state.contracts.insert(address, Deployed { vm: TestVM::new(), dispatch });
        drop(state);
        Contract::new(self.clone(), address)
    }

    /// The SDK test VM of the contract at `address`, if one is deployed.
    #[must_use]
    pub fn vm_of(&self, address: Address) -> Option<TestVM> {
        self.state
            .borrow()
            .contracts
            .get(&address)
            .map(|deployed| deployed.vm.clone())
    }

    /// Returns `true` if `address` has code.
    #[must_use]
    pub fn has_code(&self, address: Address) -> bool {
        self.state.borrow().chain.code_size(address) > 0
    }

    /// Returns the balance of `address`.
    #[must_use]
    pub fn balance(&self, address: Address) -> U256 {
        self.state.borrow().chain.balance(address)
    }

    /// Sets the balance of `address`.
    pub fn set_balance(&self, address: Address, balance: U256) {
        self.state.borrow().chain.set_balance(address, balance);
    }

    /// Credits `amount` wei to `address`.
    pub fn fund(&self, address: Address, amount: U256) {
        let balance = self.balance(address) + amount;
        self.set_balance(address, balance);
    }

    /// Sets the block number.
    pub fn set_block_number(&self, number: u64) {
        self.state.borrow().chain.set_block_number(number);
    }

    /// Returns the block number.
    #[must_use]
    pub fn block_number(&self) -> u64 {
        self.state.borrow().chain.block_number()
    }

    /// Sets the block timestamp.
    pub fn set_block_timestamp(&self, timestamp: u64) {
        self.state.borrow().chain.set_block_timestamp(timestamp);
    }

    /// Returns the block timestamp.
    #[must_use]
    pub fn block_timestamp(&self) -> u64 {
        self.state.borrow().chain.block_timestamp()
    }

    /// Sets the chain id.
    pub fn set_chain_id(&self, chain_id: u64) {
        self.state.borrow().chain.set_chain_id(chain_id);
    }

    /// Returns the chain id.
    #[must_use]
    pub fn chain_id(&self) -> u64 {
        self.state.borrow().chain.chain_id()
    }

    /// Sets `tx.origin`. By default it is the sender of the top-level call.
    pub fn set_tx_origin(&self, origin: Address) {
        self.state.borrow().chain.set_tx_origin(origin);
    }

    /// Returns every log emitted so far, in order.
    #[must_use]
    pub fn logs(&self) -> Vec<Log> {
        self.state.borrow().logs.clone()
    }

    /// Returns a raw storage word of the contract at `contract`.
    #[must_use]
    pub fn storage(&self, contract: Address, slot: U256) -> B256 {
        self.vm_of(contract).map_or(B256::ZERO, |vm| vm.get_storage(slot))
    }

    /// Describes `address` for messages: its test name, if any, and the
    /// address.
    #[must_use]
    pub fn name_of(&self, address: Address) -> String {
        match self.state.borrow().names.get(&address) {
            Some(name) => format!("`{name}` ({address})"),
            None => address.to_string(),
        }
    }

    /// Starts a top-level call from `sender` to `this` sending `value`.
    ///
    /// Records the checkpoint used by [`Env::revert_pending`].
    ///
    /// # Panics
    ///
    /// If `sender` cannot pay `value`.
    pub(crate) fn begin_call(&self, sender: Address, this: Address, value: U256) {
        let mut state = self.state.borrow_mut();
        state.frames.clear();
        state.pending = Some(state.checkpoint());
        state.last_call = Some((sender, this));
        state.chain.set_sender(sender);
        if state.transfer(sender, this, value).is_err() {
            drop(state);
            panic!(
                "{} cannot pay {value} wei to {}",
                self.name_of(sender),
                self.name_of(this)
            );
        }
        state.frames.push(Frame {
            sender,
            this,
            value,
            is_static: false,
            static_violation: false,
        });
    }

    /// Ends the top-level call.
    pub(crate) fn end_call(&self) {
        self.state.borrow_mut().frames.clear();
    }

    /// Rolls back the state changes of the last top-level call.
    ///
    /// # Panics
    ///
    /// If no call was made, or if the last call was already reverted.
    pub(crate) fn revert_pending(&self) {
        let checkpoint = self.state.borrow_mut().pending.take();
        match checkpoint {
            Some(checkpoint) => self.state.borrow_mut().restore(checkpoint),
            None => panic!("there is no call to revert"),
        }
    }

    /// Message for a call that should have reverted.
    pub(crate) fn unexpected_success(&self) -> String {
        match self.state.borrow().last_call {
            Some((sender, this)) => format!(
                "expected the call from {} to {} to revert, but it succeeded",
                self.name_of(sender),
                self.name_of(this)
            ),
            None => "expected the call to revert, but it succeeded".to_owned(),
        }
    }

    pub(crate) fn push(&self, frame: Frame) {
        self.state.borrow_mut().frames.push(frame);
    }

    pub(crate) fn pop(&self) -> Frame {
        self.state.borrow_mut().frames.pop().expect("frame stack underflow")
    }

    /// Performs a cross-contract call issued by the executing frame.
    ///
    /// Returns the call outcome; the return or revert data is also stored as
    /// the environment's return data.
    pub(crate) fn call(
        &self,
        kind: CallKind,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> ArbResult {
        let (caller, dispatch) = {
            let state = self.state.borrow();
            let caller = state.frame().clone();
            let dispatch =
                state.contracts.get(&to).and_then(|d| d.dispatch.clone());
            (caller, dispatch)
        };
        if caller.is_static && !value.is_zero() {
            return Err(Vec::new());
        }
        let checkpoint = self.state.borrow().checkpoint();
        let result = match kind {
            CallKind::Call | CallKind::StaticCall => {
                if self
                    .state
                    .borrow_mut()
                    .transfer(caller.this, to, value)
                    .is_err()
                {
                    Err(Vec::new())
                } else if let Some(dispatch) = dispatch {
                    self.push(Frame {
                        sender: caller.this,
                        this: to,
                        value,
                        is_static: caller.is_static
                            || kind == CallKind::StaticCall,
                        static_violation: false,
                    });
                    let result = dispatch(data, self.vm());
                    let frame = self.pop();
                    if frame.static_violation {
                        Err(Vec::new())
                    } else {
                        result
                    }
                } else {
                    self.mocked(kind, caller.this, to, data, value)
                }
            }
            CallKind::DelegateCall => {
                if let Some(dispatch) = dispatch {
                    self.push(Frame {
                        sender: caller.sender,
                        this: caller.this,
                        value: caller.value,
                        is_static: caller.is_static,
                        static_violation: false,
                    });
                    let result = dispatch(data, self.vm());
                    let frame = self.pop();
                    if frame.static_violation {
                        Err(Vec::new())
                    } else {
                        result
                    }
                } else {
                    self.mocked(kind, caller.this, to, data, value)
                }
            }
        };
        if result.is_err() {
            self.state.borrow_mut().restore(checkpoint);
        }
        self.state.borrow_mut().return_data = match &result {
            Ok(data) | Err(data) => data.clone(),
        };
        result
    }

    /// Answers a call to an address without a deployed contract from the
    /// mocks of the calling contract's test VM.
    ///
    /// An address without a mock behaves like an externally owned account:
    /// it accepts the value and returns no data.
    fn mocked(
        &self,
        kind: CallKind,
        caller: Address,
        to: Address,
        data: Vec<u8>,
        value: U256,
    ) -> ArbResult {
        let Some(vm) = self.vm_of(caller) else {
            return Ok(Vec::new());
        };
        let mocks = vm.snapshot();
        let mock = match kind {
            CallKind::Call => mocks.call_returns.get(&(to, data, value)),
            CallKind::StaticCall => mocks.static_call_returns.get(&(to, data)),
            CallKind::DelegateCall => {
                mocks.delegate_call_returns.get(&(to, data))
            }
        };
        mock.cloned().unwrap_or_else(|| Ok(Vec::new()))
    }
}

/// Derives a deterministic address from a kind and a name.
fn derive_address(kind: &str, name: &str) -> Address {
    let hash = keccak256(format!("openzeppelin-stylus-testing:{kind}:{name}"));
    Address::from_slice(&hash[12..])
}
