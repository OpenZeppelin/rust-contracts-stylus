#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate openzeppelin_crypto;

fuzz_target!(|data: (Vec<u8>, [u8; 32], [u8; 32])| {
    let (proof, root, leaf) = data;

    // Ensure the proof_data length is a multiple of 32
    let proof = proof
        .chunks_exact(32)
        .map(|chunk| {
            <[u8; 32]>::try_from(chunk).expect("Chunk size is always 32")
        })
        .collect::<Vec<_>>();

    _ = openzeppelin_crypto::merkle::Verifier::verify(&proof, root, leaf);
});
