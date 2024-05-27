//! Shims that mock common host imports in Stylus `wasm` programs.
//!
//! Most of the documentation is taken from the [Stylus source].
//!
//! [Stylus source]: https://github.com/OffchainLabs/stylus/blob/484efac4f56fb70f96d4890748b8ec2543d88acd/arbitrator/wasm-libraries/user-host-trait/src/lib.rs
//!
//! We allow unsafe here because safety is guaranteed by the Stylus team.
//!
//! ## Motivation
//!
//! Without these shims we can't currently run unit tests for stylus contracts,
//! since the symbols the compiled binaries expect to find are not there.
//!
//! If you run `cargo test` on a fresh Stylus project, it will error with:
//!
//! ```terminal
//! dyld[97792]: missing symbol called
//! ```
//!
//! This crate is a temporary solution until the Stylus team provides us with a
//! different and more stable mechanism for unit-testing our contracts.
//!
//! ## Usage
//!
//! Import these shims in your test modules as `grip::prelude::*` to populate
//! the namespace with the appropriate symbols.
//!
//! ```rust,ignore
//! #[cfg(test)]
//! mod tests {
//!     use contracts::erc20::ERC20;
//!
//!     #[grip::test]
//!     fn reads_balance(contract: ERC20) {
//!         let balance = contract.balance_of(Address::ZERO); // Access storage.
//!         assert_eq!(balance, U256::ZERO);
//!     }
//! }
//! ```
//!
//! Note that for proper usage, tests should have exclusive access to storage,
//! since they run in parallel, which may cause undesired results.
//!
//! One solution is to wrap tests with a function that acquires a global mutex:
//!
//! ```rust,no_run
//! use std::sync::{Mutex, MutexGuard};
//!
//! use grip::prelude::reset_storage;
//!
//! pub static STORAGE_MUTEX: Mutex<()> = Mutex::new(());
//!
//! pub fn acquire_storage() -> MutexGuard<'static, ()> {
//!     STORAGE_MUTEX.lock().unwrap()
//! }
//!
//! pub fn with_context<C: Default>(closure: impl FnOnce(&mut C)) {
//!     let _lock = acquire_storage();
//!     let mut contract = C::default();
//!     closure(&mut contract);
//!     reset_storage();
//! }
//!
//! #[test]
//! fn reads_balance() {
//!     grid::context::with_context::<ERC20>(|token| {
//!         let balance = token.balance_of(Address::ZERO);
//!         assert_eq!(balance, U256::ZERO);
//!     })
//! }
//! ```
#![allow(clippy::missing_safety_doc)]
use std::slice;

use tiny_keccak::{Hasher, Keccak};

use crate::storage::{read_bytes32, write_bytes32, STORAGE};

pub(crate) const WORD_BYTES: usize = 32;
pub(crate) type Bytes32 = [u8; WORD_BYTES];

/// Efficiently computes the [`keccak256`] hash of the given preimage.
/// The semantics are equivalent to that of the EVM's [`SHA3`] opcode.
///
/// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
/// [`SHA3`]: https://www.evm.codes/#20
#[no_mangle]
pub unsafe extern "C" fn native_keccak256(
    bytes: *const u8,
    len: usize,
    output: *mut u8,
) {
    let mut hasher = Keccak::v256();

    let data = unsafe { slice::from_raw_parts(bytes, len) };
    hasher.update(data);

    let output = unsafe { slice::from_raw_parts_mut(output, WORD_BYTES) };
    hasher.finalize(output);
}

/// Reads a 32-byte value from permanent storage. Stylus's storage format is
/// identical to that of the EVM. This means that, under the hood, this hostio
/// is accessing the 32-byte value stored in the EVM state trie at offset
/// `key`, which will be `0` when not previously set. The semantics, then, are
/// equivalent to that of the EVM's [`SLOAD`] opcode.
///
/// [`SLOAD`]: https://www.evm.codes/#54
#[no_mangle]
pub unsafe extern "C" fn storage_load_bytes32(key: *const u8, out: *mut u8) {
    let key = unsafe { read_bytes32(key) };

    let value = STORAGE
        .lock()
        .unwrap()
        .get(&key)
        .map(Bytes32::to_owned)
        .unwrap_or_default();

    unsafe { write_bytes32(out, value) };
}

// TODO: change it when 0.5.0 release will work with a nitro test node
/// Stores a 32-byte value to permanent storage. Stylus's storage format is
/// identical to that of the EVM. This means that, under the hood, this hostio
/// is storing a 32-byte value into the EVM state trie at offset `key`.
/// Furthermore, refunds are tabulated exactly as in the EVM. The semantics,
/// then, are equivalent to that of the EVM's [`SSTORE`] opcode.
///
/// Note: we require the [`SSTORE`] sentry per EVM rules. The `gas_cost`
/// returned by the EVM API may exceed this amount, but that's ok because the
/// predominant cost is due to state bloat concerns.
///
/// [`SSTORE`]: https://www.evm.codes/#55
#[no_mangle]
pub unsafe extern "C" fn storage_cache_bytes32(
    key: *const u8,
    value: *const u8,
) {
    let (key, value) = unsafe { (read_bytes32(key), read_bytes32(value)) };
    STORAGE.lock().unwrap().insert(key, value);
}
/// Persists any dirty values in the storage cache to the EVM state trie,
/// dropping the cache entirely if requested. Analogous to repeated invocations
/// of [`SSTORE`].
///
/// [`SSTORE`]: https://www.evm.codes/#55
pub fn storage_flush_cache(_: bool) {
    // No-op: we don't use the cache in our unit-tests.
}

/// Dummy msg sender set for tests.
pub const MSG_SENDER: &[u8; 42] = b"0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF";

/// Gets the address of the account that called the program. For normal
/// L2-to-L2 transactions the semantics are equivalent to that of the EVM's
/// [`CALLER`] opcode, including in cases arising from [`DELEGATE_CALL`].
///
/// For L1-to-L2 retryable ticket transactions, the top-level sender's address
/// will be aliased. See [`Retryable Ticket Address Aliasing`][aliasing] for
/// more information on how this works.
///
/// [`CALLER`]: https://www.evm.codes/#33
/// [`DELEGATE_CALL`]: https://www.evm.codes/#f4
/// [aliasing]: https://developer.arbitrum.io/arbos/l1-to-l2-messaging#address-aliasing
#[no_mangle]
pub unsafe extern "C" fn msg_sender(sender: *mut u8) {
    let addr = const_hex::const_decode_to_array::<20>(MSG_SENDER).unwrap();
    std::ptr::copy(addr.as_ptr(), sender, 20);
}

/// Emits an EVM log with the given number of topics and data, the first bytes
/// of which should be the 32-byte-aligned topic data. The semantics are
/// equivalent to that of the EVM's [`LOG0`], [`LOG1`], [`LOG2`], [`LOG3`], and
/// [`LOG4`] opcodes based on the number of topics specified. Requesting more
/// than `4` topics will induce a revert.
///
/// [`LOG0`]: https://www.evm.codes/#a0
/// [`LOG1`]: https://www.evm.codes/#a1
/// [`LOG2`]: https://www.evm.codes/#a2
/// [`LOG3`]: https://www.evm.codes/#a3
/// [`LOG4`]: https://www.evm.codes/#a4
#[no_mangle]
pub unsafe extern "C" fn emit_log(_: *const u8, _: usize, _: usize) {
    // No-op: we don't check for events in our unit-tests.
}
