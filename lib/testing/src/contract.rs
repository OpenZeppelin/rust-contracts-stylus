//! Handle to a contract deployed in an [`Env`].
use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolEvent;
use stylus_sdk::{storage::StorageType, testing::TestVM};

use crate::env::{Env, Log};

/// A contract deployed in an [`Env`].
///
/// Methods are called through [`Contract::sender`], which runs them as a call
/// from the given account against the contract's own storage.
pub struct Contract<T> {
    env: Env,
    address: Address,
    _marker: PhantomData<T>,
}

impl<T> Debug for Contract<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Contract")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

impl<T: StorageType> Contract<T> {
    pub(crate) fn new(env: Env, address: Address) -> Self {
        Self { env, address, _marker: PhantomData }
    }

    /// The address of this contract.
    #[must_use]
    pub fn address(&self) -> Address {
        self.address
    }

    /// The environment this contract lives in.
    #[must_use]
    pub fn env(&self) -> &Env {
        &self.env
    }

    /// The SDK test VM that holds this contract's storage and call mocks.
    ///
    /// Use it to inspect raw storage or to mock calls this contract makes to
    /// addresses that are not deployed in the environment.
    #[must_use]
    pub fn vm(&self) -> TestVM {
        self.env.vm_of(self.address).expect("the contract should be deployed")
    }

    /// The ETH balance of this contract.
    #[must_use]
    pub fn balance(&self) -> U256 {
        self.env.balance(self.address)
    }

    /// Credits `amount` wei to this contract.
    pub fn fund(&self, amount: U256) {
        self.env.fund(self.address, amount);
    }

    /// Prepares a call from `sender` with no value.
    ///
    /// The returned handle dereferences to the contract: every method called
    /// through it runs as a call from `sender`. A method that returns `Err`
    /// keeps its state changes; use
    /// [`ResultExt`](crate::ResultExt) to revert them.
    pub fn sender(&self, sender: impl Into<Address>) -> ContractCall<'_, T> {
        self.sender_and_value(sender, U256::ZERO)
    }

    /// Prepares a call from `sender` that sends `value` wei.
    ///
    /// # Panics
    ///
    /// When a method is called and `sender` cannot pay `value`.
    pub fn sender_and_value(
        &self,
        sender: impl Into<Address>,
        value: impl Into<U256>,
    ) -> ContractCall<'_, T> {
        ContractCall {
            contract: self,
            sender: sender.into(),
            value: value.into(),
            instance: self.instance(),
        }
    }

    /// Logs emitted by this contract so far, in order.
    #[must_use]
    pub fn logs(&self) -> Vec<Log> {
        self.env
            .logs()
            .into_iter()
            .filter(|log| log.emitter == self.address)
            .collect()
    }

    /// Returns `true` if this contract emitted `event`.
    ///
    /// Events are compared by their ABI encoding (topics and data).
    pub fn emitted<E: SolEvent>(&self, event: &E) -> bool {
        self.logs().iter().any(|log| log.matches(event))
    }

    /// Asserts that this contract emitted `event`.
    ///
    /// # Panics
    ///
    /// If the event was not emitted. The message lists the emitted events of
    /// the same type.
    #[track_caller]
    pub fn assert_emitted<E: SolEvent + Debug>(&self, event: &E) {
        if self.emitted(event) {
            return;
        }
        let same_kind: Vec<E> =
            self.logs().iter().filter_map(Log::decode::<E>).collect();
        panic!(
            "{} did not emit {event:?}; matching events: {same_kind:?}",
            self.env.name_of(self.address)
        );
    }

    fn instance(&self) -> T {
        unsafe { T::new(U256::ZERO, 0, self.env.vm()) }
    }
}

/// A pending call to a [`Contract`] from a chosen sender.
///
/// Dereferencing it starts the call: it records a checkpoint for
/// [`ResultExt`](crate::ResultExt), transfers the value and makes the
/// contract see the sender, value and its own storage until the handle is
/// dropped.
pub struct ContractCall<'a, T: StorageType> {
    contract: &'a Contract<T>,
    sender: Address,
    value: U256,
    instance: T,
}

impl<T: StorageType> ContractCall<'_, T> {
    fn begin(&self) {
        self.contract.env.begin_call(
            self.sender,
            self.contract.address,
            self.value,
        );
    }
}

impl<T: StorageType> Deref for ContractCall<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.begin();
        &self.instance
    }
}

impl<T: StorageType> DerefMut for ContractCall<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.begin();
        &mut self.instance
    }
}

impl<T: StorageType> Drop for ContractCall<'_, T> {
    fn drop(&mut self) {
        self.contract.env.end_call();
    }
}
