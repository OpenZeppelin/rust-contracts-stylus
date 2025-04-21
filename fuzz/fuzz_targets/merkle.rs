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

        if let Ok(multi_verif) = multi_verif {
            // ensure the results match if no errors occurred
            assert_eq!(single_verif, multi_verif);
        } else {
            // otherwise single-proof verification must be false
            assert!(!single_verif);
        }
    }
});
