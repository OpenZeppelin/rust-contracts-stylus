#![no_main]
use libfuzzer_sys::fuzz_target;
use openzeppelin_crypto::merkle::Verifier;
use rs_merkle::{MerkleTree, algorithms::Keccak256, Hasher};

fuzz_target!(|data: &[u8]| {
    if data.len() < 33 {
        return;
    }

    
    let mut leaves = Vec::new();
    for chunk in data[1..].chunks(32) {
        if chunk.len() == 32 {
            let mut leaf = [0u8; 32];
            leaf.copy_from_slice(chunk);
            leaves.push(leaf);
        }
    }

    if leaves.len() < 2 {
        return;
    }

    println!("Leaves: {:?}", leaves);

    
    let rs_merkle_tree = MerkleTree::<Keccak256>::from_leaves(&leaves);
    let rs_root:[u8; 32] = rs_merkle_tree.root().unwrap();
    let index_to_prove = data[0] as usize % leaves.len();
    println!("index to prove {}", index_to_prove);
    let rs_proof = rs_merkle_tree.proof(&[index_to_prove]);

    
    let oz_proof: Vec<[u8; 32]> = rs_proof.proof_hashes().iter().map(|h| h.to_owned()).collect();
    println!("Proof: {:?}", oz_proof);

    
    let oz_verification = Verifier::verify(&oz_proof, rs_root, leaves[index_to_prove]);

    
    assert_eq!(
        oz_verification,
        rs_proof.verify(rs_root, &[index_to_prove], &[leaves[index_to_prove]], leaves.len()),
        "Verification mismatch between rs-merkle and OpenZeppelin"
    );
});


