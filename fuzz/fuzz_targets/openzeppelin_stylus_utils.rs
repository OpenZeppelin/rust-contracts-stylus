#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate openzeppelin_stylus;

use alloy_primitives::B256;
pub use motsu::prelude::*;

struct Storage;

unsafe impl stylus_sdk::prelude::TopLevelStorage for Storage {}

fuzz_target!(|data: (B256, u8, B256, B256)| {
    let (hash, v, r, s) = data;

    let mut storage = Storage;

    _ = openzeppelin_stylus::utils::cryptography::ecdsa::recover(
        &mut storage,
        hash,
        v,
        r,
        s,
    );
});

// TODO: fuzzing code from [`openzeppelin_stylus`] will be possible once the
// Stylus team refactors the interaction with Host.
// See: https://github.com/OffchainLabs/stylus-sdk-rs/pull/195
