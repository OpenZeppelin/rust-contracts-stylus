#![no_main]

use libfuzzer_sys::fuzz_target;
use openzeppelin_crypto::merkle::Verifier;
use test_fuzz::Input;

fuzz_target!(|input: Input| {
    let Input { root, leaves, proof, proof_flags } = input;

    let multi_verif =
        Verifier::verify_multi_proof(&proof, &proof_flags, root, &leaves);

    // If we have a single leaf, also test the regular verification
    if leaves.len() == 1 {
        let single_verif = Verifier::verify(&proof, root, leaves[0]);

        // ensure the results match if no errors occurred
        if let Ok(multi_verif) = multi_verif {
            assert_eq!(single_verif, multi_verif);
        }

        // the reason we don't make any assumptions in case of multi-proof
        // errors is that it is possible that fuzzer generates invalid
        // proof_flags for valid merkle tree, returning an error
    }
});
