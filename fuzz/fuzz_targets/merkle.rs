#![no_main]

use std::ops::Deref;

use libfuzzer_sys::{
    arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured},
    fuzz_target,
};
use openzeppelin_crypto::merkle::Verifier;
use rs_merkle::{algorithms::Keccak256, Hasher, MerkleTree};
use test_fuzz::{
    consts::merkle::{MAX_LEAVES, MIN_INDICES, MIN_LEAVES},
    CommutativeKeccak256,
};

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

        let idx_range = MIN_INDICES..=(leaves.len() - 1);
        let num_indices = u.int_in_range(idx_range)?;
        let mut indices_to_prove = Vec::with_capacity(num_indices);
        for _ in 0..num_indices {
            let mut idx = u.arbitrary::<usize>()? % leaves.len();
            while indices_to_prove.contains(&idx) {
                idx = (idx + 1) % leaves.len();
            }
            indices_to_prove.push(idx);
        }

        let proof_flags = u.arbitrary()?;

        Ok(Input { leaves, indices_to_prove, proof_flags })
    }
}

// We construct a valid Merkle tree using [`rs_merkle`] and make various
// assertions against our own implementation.
fuzz_target!(|input: Input| {
    let Input { leaves, indices_to_prove, proof_flags } = input;

    let merkle_tree = MerkleTree::<CommutativeKeccak256>::from_leaves(&leaves);
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

    let oz_proof = proof.proof_hashes();
    let oz_verified = Verifier::verify(oz_proof, root, leaves[index_to_prove]);

    // Both should be true
    assert_eq!(oz_verified, rs_verified);

    // ===== TEST 2: Multi-proof differential testing =====

    if indices_to_prove.len() > 1 {
        // Create a subset of leaves to verify
        let leaf_subset: Vec<[u8; 32]> =
            indices_to_prove.iter().map(|&i| leaves[i]).collect();

        // Get a multi-proof from rs_merkle
        let multi_proof = merkle_tree.proof(&indices_to_prove);
        let oz_multi_proof = multi_proof.proof_hashes();

        // Create proper flags for multiple leaves
        let appropriate_proof_flags = if proof_flags.len()
            == leaf_subset.len() + oz_multi_proof.len() - 1
        {
            proof_flags.clone()
        } else {
            // Create a dummy set of flags with appropriate length
            vec![false; leaf_subset.len() + oz_multi_proof.len() - 1]
        };
        if appropriate_proof_flags.len() > 0 {
            // Just test that the function doesn't panic
            _ = Verifier::verify_multi_proof(
                oz_multi_proof,
                &appropriate_proof_flags,
                root,
                &leaf_subset,
            );
        }
    }
});
