#![no_main]

use libfuzzer_sys::fuzz_target;
use openzeppelin_crypto::merkle::Verifier;
use test_fuzz::Input;

fuzz_target!(|input: Input| {
    let Input { root, leaves, proof, proof_flags } = input;

    _ = Verifier::verify_multi_proof(&proof, &proof_flags, root, &leaves);
});
