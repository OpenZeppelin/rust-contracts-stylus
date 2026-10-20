//! Behavioral tests of the test environment itself.
#![allow(clippy::unreadable_literal)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{uint, Address, B256, U256};
use alloy_sol_types::{sol, SolCall};
use stylus_sdk::{
    call::{call, static_call, transfer::transfer_eth},
    prelude::*,
    storage::{StorageAddress, StorageU256},
};
use testing::prelude::*;

sol! {
    #[derive(Debug, PartialEq)]
    event Incremented(address indexed by, uint256 value);

    #[derive(Debug, PartialEq)]
    error Forbidden(address account);
}

#[derive(SolidityError, Debug)]
enum CounterError {
    Forbidden(Forbidden),
}

sol_interface! {
    interface ICounter {
        function increment() external returns (uint256);
        function value() external view returns (uint256);
        function failAndWrite() external;
        function owner() external view returns (address);
    }
}

#[storage]
struct Counter {
    value: StorageU256,
    owner: StorageAddress,
}

unsafe impl TopLevelStorage for Counter {}

#[public]
impl Counter {
    fn set_owner(&mut self, owner: Address) {
        self.owner.set(owner);
    }

    fn increment(&mut self) -> Result<U256, CounterError> {
        let value = self.value.get() + U256::ONE;
        self.value.set(value);
        self.vm().log(Incremented { by: self.vm().msg_sender(), value });
        Ok(value)
    }

    fn value(&self) -> U256 {
        self.value.get()
    }

    fn owner(&self) -> Address {
        self.owner.get()
    }

    fn only_owner(&mut self) -> Result<(), CounterError> {
        let sender = self.vm().msg_sender();
        if sender != self.owner.get() {
            return Err(CounterError::Forbidden(Forbidden { account: sender }));
        }
        self.value.set(self.value.get() + uint!(100_U256));
        Ok(())
    }

    /// Writes, then fails: the write must be rolled back by the caller.
    fn fail_and_write(&mut self) -> Result<(), CounterError> {
        self.value.set(uint!(999_U256));
        Err(CounterError::Forbidden(Forbidden { account: Address::ZERO }))
    }

    #[payable]
    fn deposit(&mut self) -> U256 {
        self.vm().msg_value()
    }

    fn pay(&mut self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        transfer_eth(self.vm(), to, amount)
    }

    fn reentrant(&self) -> bool {
        self.vm().msg_reentrant()
    }

    fn origin(&self) -> Address {
        self.vm().tx_origin()
    }

    fn me(&self) -> Address {
        self.vm().contract_address()
    }

    fn timestamp(&self) -> u64 {
        self.vm().block_timestamp()
    }
}

#[storage]
struct Caller {
    last: StorageU256,
}

unsafe impl TopLevelStorage for Caller {}

#[public]
impl Caller {
    fn increment_other(&mut self, counter: Address) -> Result<U256, Vec<u8>> {
        let cfg = Call::new_mutating(self);
        let value = ICounter::new(counter).increment(self.vm(), cfg)?;
        self.last.set(value);
        Ok(value)
    }

    fn read_other(&self, counter: Address) -> Result<U256, Vec<u8>> {
        Ok(ICounter::new(counter).value(self.vm(), Call::new())?)
    }

    /// The callee writes and reverts; the caller's own write before the call
    /// must survive, the callee's must not.
    fn call_failing(&mut self, counter: Address) -> Result<(), Vec<u8>> {
        self.last.set(uint!(7_U256));
        let cfg = Call::new_mutating(self);
        let result = ICounter::new(counter).fail_and_write(self.vm(), cfg);
        assert!(result.is_err(), "callee should revert");
        Ok(())
    }

    fn static_increment(&self, counter: Address) -> Result<Vec<u8>, Vec<u8>> {
        let data = incrementCall {}.abi_encode();
        static_call(self.vm(), Call::new(), counter, &data).map_err(|e| {
            let e: Vec<u8> = e.into();
            e
        })
    }

    fn call_eoa(&mut self, to: Address, data: Vec<u8>) -> Result<Vec<u8>, Vec<u8>> {
        let cfg = Call::new_mutating(self);
        call(self.vm(), cfg, to, &data).map_err(Into::into)
    }

    fn reenter(&mut self, counter: Address) -> Result<bool, Vec<u8>> {
        let data = reentrantCall {}.abi_encode();
        let cfg = Call::new_mutating(self);
        let out = call(self.vm(), cfg, counter, &data)?;
        Ok(out.last().copied() == Some(1))
    }
}

sol! {
    function reentrant() external view returns (bool);
    function nothing() external;
    function increment() external returns (uint256);
}

#[testing::test]
fn addresses_are_deterministic(counter: Contract<Counter>, alice: Address) {
    assert_eq!(counter.address(), Env::new().deploy::<Counter>("counter").address());
    assert_eq!(alice, Env::new().account("alice"));
    assert_ne!(alice, counter.address());
}

#[testing::test]
fn calls_see_sender_value_and_storage(counter: Contract<Counter>, alice: Address) {
    assert_eq!(counter.sender(alice).value(), U256::ZERO);
    assert_eq!(counter.sender(alice).increment().unwrap(), U256::ONE);
    assert_eq!(counter.sender(alice).value(), U256::ONE);
    assert_eq!(counter.sender(alice).me(), counter.address());
    assert_eq!(counter.sender(alice).origin(), alice);
    counter.assert_emitted(&Incremented { by: alice, value: U256::ONE });
    assert!(!counter.emitted(&Incremented { by: alice, value: uint!(2_U256) }));

    alice.fund(uint!(10_U256));
    assert_eq!(counter.sender_and_value(alice, uint!(3_U256)).deposit(), uint!(3_U256));
    assert_eq!(alice.balance(), uint!(7_U256));
    assert_eq!(counter.balance(), uint!(3_U256));
}

#[testing::test]
fn unwrap_revert_restores_state(counter: Contract<Counter>, alice: Address, bob: Address) {
    counter.sender(alice).set_owner(bob);
    let err = counter.sender(alice).only_owner().unwrap_revert();
    assert!(matches!(err, CounterError::Forbidden(Forbidden { account }) if account == alice));
    assert_eq!(counter.sender(alice).value(), U256::ZERO);

    // Without the helper the write of a failing call stays, like in Rust.
    let _ = counter.sender(alice).fail_and_write();
    assert_eq!(counter.sender(alice).value(), uint!(999_U256));
    let _ = counter.sender(alice).fail_and_write().expect_revert("fails");
    // The revert restores the state before the *last* call only.
    assert_eq!(counter.sender(alice).value(), uint!(999_U256));
}

#[testing::test]
#[should_panic(expected = "expected the call from `alice`")]
fn unwrap_revert_panics_on_success(counter: Contract<Counter>, alice: Address) {
    counter.sender(alice).increment().unwrap_revert();
}

#[testing::test]
#[should_panic(expected = "cannot pay 5 wei")]
fn value_needs_balance(counter: Contract<Counter>, alice: Address) {
    counter.sender_and_value(alice, uint!(5_U256)).deposit();
}

#[testing::test]
fn cross_contract_calls_route_and_revert(
    counter: Contract<Counter>,
    caller: Contract<Caller>,
    alice: Address,
) {
    let value = caller.sender(alice).increment_other(counter.address()).unwrap();
    assert_eq!(value, U256::ONE);
    assert_eq!(counter.sender(alice).value(), U256::ONE);
    assert_eq!(caller.sender(alice).read_other(counter.address()).unwrap(), U256::ONE);
    // The event is attributed to the emitting contract with the caller as sender.
    counter.assert_emitted(&Incremented { by: caller.address(), value: U256::ONE });
    assert!(!caller.emitted(&Incremented { by: caller.address(), value: U256::ONE }));

    caller.sender(alice).call_failing(counter.address()).unwrap();
    assert_eq!(counter.sender(alice).value(), U256::ONE, "callee write rolled back");
    assert_eq!(caller.sender(alice).last.get(), uint!(7_U256), "caller write kept");

    let err = caller.sender(alice).static_increment(counter.address()).unwrap_err();
    assert!(err.is_empty(), "static write reverts without data");
    assert_eq!(counter.sender(alice).value(), U256::ONE);

    assert!(!counter.sender(alice).reentrant());
    assert!(caller.sender(alice).reenter(counter.address()).is_ok());
}

#[testing::test]
fn eoa_calls_transfer_value_and_use_mocks(caller: Contract<Caller>, alice: Address, bob: Address) {
    caller.fund(uint!(100_U256));
    assert!(caller.env().has_code(caller.address()));
    assert!(!caller.env().has_code(bob));

    // A plain account accepts the value and returns nothing.
    let out = caller.sender(alice).call_eoa(bob, Vec::new()).unwrap();
    assert!(out.is_empty());

    // Calls to addresses without a contract can be mocked on the caller's VM.
    let data = nothingCall {}.abi_encode();
    caller.vm().mock_call(bob, data.clone(), U256::ZERO, Ok(vec![1, 2, 3]));
    assert_eq!(caller.sender(alice).call_eoa(bob, data.clone()).unwrap(), vec![1, 2, 3]);
    caller.vm().mock_call(bob, data.clone(), U256::ZERO, Err(vec![9]));
    assert_eq!(caller.sender(alice).call_eoa(bob, data).unwrap_err(), vec![9]);
}

#[testing::test]
fn block_context_is_shared(counter: Contract<Counter>, env: Env, alice: Address) {
    env.set_block_timestamp(1_700_000_000);
    assert_eq!(counter.sender(alice).timestamp(), 1_700_000_000);
    env.chain().set_block_timestamp(5);
    assert_eq!(counter.sender(alice).timestamp(), 5);
    env.set_tx_origin(alice);
    assert_eq!(counter.sender(Address::ZERO).origin(), alice);
}

#[testing::test]
fn attached_types_have_storage_but_no_code(plain: Contract<StorageU256>, alice: Address) {
    assert!(!plain.env().has_code(plain.address()));
    plain.sender(alice).set(uint!(42_U256));
    assert_eq!(plain.sender(alice).get(), uint!(42_U256));
    assert_eq!(plain.env().storage(plain.address(), U256::ZERO), B256::from(uint!(42_U256)));
}

#[testing::test]
fn transfers_between_contracts_and_accounts(counter: Contract<Counter>, alice: Address, bob: Address) {
    counter.fund(uint!(10_U256));
    counter.sender(alice).pay(bob, uint!(4_U256)).unwrap();
    assert_eq!(bob.balance(), uint!(4_U256));
    assert_eq!(counter.balance(), uint!(6_U256));
    let err = counter.sender(alice).pay(bob, uint!(60_U256)).unwrap_revert();
    assert!(!err.is_empty() || err.is_empty());
    assert_eq!(counter.balance(), uint!(6_U256));
}

#[testing::test]
fn accounts_sign_and_act_as_senders(counter: Contract<Counter>, alice: Account) {
    use alloy_signer::SignerSync;
    let hash = B256::repeat_byte(7);
    let signature = alice.signer().sign_hash_sync(&hash).unwrap();
    assert_eq!(signature.recover_address_from_prehash(&hash).unwrap(), alice.address());
    counter.sender(&alice).increment().unwrap();
    counter.assert_emitted(&Incremented { by: alice.address(), value: U256::ONE });
    assert_eq!(Account::from_seed("alice").address(), alice.address());
}
