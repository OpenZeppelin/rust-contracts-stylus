#![no_main]

use libfuzzer_sys::fuzz_target;
use openzeppelin_crypto::merkle::Verifier;
use test_fuzz::Input;

fuzz_target!(|input: Input| {
    let Input { root, single_proof, multi_proof } = input;

    // ===== TEST 1: Basic single-proof differential testing =====

    _ = Verifier::verify(&single_proof.proof, root, single_proof.leaf);

    // ===== TEST 2: Multi-proof differential testing =====

    _ = Verifier::verify_multi_proof(
        &multi_proof.proof,
        &multi_proof.proof_flags,
        root,
        &multi_proof.leaves,
    );
});
