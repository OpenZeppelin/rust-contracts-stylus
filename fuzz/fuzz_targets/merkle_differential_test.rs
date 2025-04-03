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
const BITS_PER_LEAF: usize = 32;

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

        let length = num_of_leaves * BITS_PER_LEAF;

        let mut vec = Vec::with_capacity(length);
        for _ in 0..length {
            let hash = Keccak256::hash(u.arbitrary()?);
            vec.push(hash);
        }

        Ok(Leaves(vec))
    }
}

#[derive(Debug)]
struct Input {
    leaves: Leaves,
    index_to_prove: usize,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let leaves: Leaves = u.arbitrary()?;
        let index_to_prove = u.int_in_range(0..=(leaves.len() - 1))?;

        Ok(Input { leaves, index_to_prove })
    }
}

fuzz_target!(|input: Input| {
    let Input { leaves, index_to_prove } = input;

    let merkle_tree = MerkleTree::<Keccak256>::from_leaves(&leaves);
    let root = merkle_tree.root().expect("root should be present");
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
});
