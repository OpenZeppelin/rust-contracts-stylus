#![no_main]

use std::ops::Deref;

use libfuzzer_sys::{
    arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured},
    fuzz_target,
};
use openzeppelin_crypto::merkle::Verifier;
use rs_merkle::{algorithms::Keccak256, Hasher, MerkleTree};

const MIN_LEAVES: usize = 2;
const MAX_LEAVES: usize = 31;

#[derive(Debug)]
struct Leaves(Vec<[u8; 32]>);

impl Deref for Leaves {
    type Target = Vec<[u8; 32]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Arbitrary<'a> for Leaves {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let num_of_leaves = u.int_in_range(MIN_LEAVES..=MAX_LEAVES)?;

        let mut vec = Vec::with_capacity(num_of_leaves);
        for _ in 0..num_of_leaves {
            let bytes = u.arbitrary()?;
            let hash = Keccak256::hash(bytes);
            vec.push(hash);
        }

        Ok(Leaves(vec))
    }
}

#[derive(Debug)]
struct Input {
    leaves: Leaves,
    indices_to_prove: Vec<usize>,
    proof_flags: Vec<bool>,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let leaves: Leaves = u.arbitrary()?;

        let num_indices = u.int_in_range(1..=std::cmp::min(3, leaves.len()))?;
        let mut indices_to_prove = Vec::with_capacity(num_indices);
        for _ in 0..num_indices {
            let mut idx = u.int_in_range(0..=(leaves.len() - 1))?;
            while indices_to_prove.contains(&idx) {
                idx = u.int_in_range(0..=(leaves.len() - 1))?;
            }
            indices_to_prove.push(idx);
        }

        let proof_flags = u.arbitrary()?;

        Ok(Input { leaves, indices_to_prove, proof_flags })
    }
}

fuzz_target!(|input: Input| {
    let Input { leaves, indices_to_prove, proof_flags } = input;

    let merkle_tree = MerkleTree::<Keccak256>::from_leaves(&leaves);
    let root = merkle_tree.root().expect("root should be present");

    // ===== TEST 1: Basic single-proof differential testing =====

    let index_to_prove = indices_to_prove[0];

    let proof = merkle_tree.proof(&[index_to_prove]);
    let rs_verified = proof.verify(
        root,
        &[index_to_prove],
        &[leaves[index_to_prove]],
        leaves.len(),
    );

    let oz_proof = proof.proof_hashes().to_vec();
    let oz_verified = Verifier::verify(&oz_proof, root, leaves[index_to_prove]);

    assert_eq!(oz_verified, rs_verified);

    // ===== TEST 2: Multi-proof verification with single leaf =====

    // For OpenZeppelin multi-proof verification, we need to prepare appropriate
    // flags Let's create reasonable flags if the provided ones are
    // inappropriate
    let appropriate_proof_flags =
        if proof_flags.len() == oz_proof.len() + leaves.len() - 1 {
            proof_flags.clone()
        } else {
            // Create a dummy set of flags with appropriate length
            vec![false; oz_proof.len() + leaves.len() - 1]
        };

    let oz_multi_single_result = Verifier::verify_multi_proof(
        &oz_proof,
        &appropriate_proof_flags,
        root,
        &[leaves[index_to_prove]],
    );

    // This may or may not be valid depending on the flags
    // Just check it doesn't panic, but verify behavior if valid
    if let Ok(oz_multi_verified) = oz_multi_single_result {
        assert_eq!(oz_multi_verified, rs_verified);
    }

    // ===== TEST 3: Testing with multiple leaves in multi-proof =====

    if indices_to_prove.len() > 1 {
        // Create a subset of leaves to verify
        let leaf_subset: Vec<[u8; 32]> =
            indices_to_prove.iter().map(|&i| leaves[i]).collect();

        // Get a multi-proof from rs_merkle
        let multi_proof = merkle_tree.proof(&indices_to_prove);
        let oz_multi_proof = multi_proof.proof_hashes();

        // Create proper flags for multiple leaves
        let multi_flags_len = leaf_subset.len() + oz_multi_proof.len() - 1;
        if multi_flags_len > 0 {
            let multi_flags = vec![false; multi_flags_len];

            // Just test that the function doesn't panic
            _ = Verifier::verify_multi_proof(
                oz_multi_proof,
                &multi_flags,
                root,
                &leaf_subset,
            );
        }
    }
});
