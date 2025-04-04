#![no_main]
use libfuzzer_sys::fuzz_target;
use openzeppelin_crypto::{
    hash::{BuildHasher, Hasher},
    keccak::KeccakBuilder,
};

fuzz_target!(|data: &[u8]| {
    let mut hasher = KeccakBuilder.build_hasher();
    hasher.update(data);
    _ = hasher.finalize();
});
