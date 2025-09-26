#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::needless_pass_by_value, clippy::unused_self)]

extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::B256;
use openzeppelin_crypto::{
    merkle::{self, Verifier},
    KeccakBuilder,
};
use stylus_sdk::{alloy_sol_types::sol, prelude::*};

sol! {
    error MerkleProofInvalidMultiProofLength();
    error MerkleProofInvalidRootChild();
    error MerkleProofInvalidTotalHashes();
    error MerkleProofNoLeaves();
}

#[derive(SolidityError)]
enum VerifierError {
    InvalidProofLength(MerkleProofInvalidMultiProofLength),
    InvalidRootChild(MerkleProofInvalidRootChild),
    InvalidTotalHashes(MerkleProofInvalidTotalHashes),
    NoLeaves(MerkleProofNoLeaves),
}

impl core::convert::From<merkle::MultiProofError> for VerifierError {
    fn from(value: merkle::MultiProofError) -> Self {
        match value {
            merkle::MultiProofError::InvalidProofLength => {
                VerifierError::InvalidProofLength(
                    MerkleProofInvalidMultiProofLength {},
                )
            }
            merkle::MultiProofError::InvalidRootChild => {
                VerifierError::InvalidRootChild(MerkleProofInvalidRootChild {})
            }
            merkle::MultiProofError::InvalidTotalHashes => {
                VerifierError::InvalidTotalHashes(
                    MerkleProofInvalidTotalHashes {},
                )
            }
            merkle::MultiProofError::NoLeaves => {
                VerifierError::NoLeaves(MerkleProofNoLeaves {})
            }
        }
    }
}

#[entrypoint]
#[storage]
struct VerifierContract;

#[public]
impl VerifierContract {
    fn verify(&self, proof: Vec<B256>, root: B256, leaf: B256) -> bool {
        let proof: Vec<[u8; 32]> = proof.into_iter().map(|m| *m).collect();
        Verifier::<KeccakBuilder>::verify(&proof, *root, *leaf)
    }

    fn verify_multi_proof(
        &self,
        proof: Vec<B256>,
        proof_flags: Vec<bool>,
        root: B256,
        leaves: Vec<B256>,
    ) -> Result<bool, VerifierError> {
        let proof: Vec<[u8; 32]> = proof.into_iter().map(|m| *m).collect();
        let leaves: Vec<[u8; 32]> = leaves.into_iter().map(|m| *m).collect();
        Ok(Verifier::<KeccakBuilder>::verify_multi_proof(
            &proof,
            &proof_flags,
            *root,
            &leaves,
        )?)
    }
}
