#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate openzeppelin_crypto;

use crate::openzeppelin_crypto::{
    hash::{BuildHasher, Hasher},
    keccak::KeccakBuilder,
};

fuzz_target!(|data: &[u8]| {
    let mut hasher = KeccakBuilder.build_hasher();

    // Test memory safety by updating with different slice patterns
    for i in 0..data.len() {
        hasher.update(&data[..i]);
        let mut new_hasher = KeccakBuilder.build_hasher();
        new_hasher.update(&data[i..]);
        _ = new_hasher.finalize();
    }

    // Finalize the original hasher
    _ = hasher.finalize();
});
