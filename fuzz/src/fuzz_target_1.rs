#![no_main]

mod abi;

use abi::ECDSA;
use alloy_primitives::B256;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (B256, u8, B256, B256)| {
    // fuzzed code goes here
});
