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
    hasher.update(data);
    _ = hasher.finalize();
});
