#![cfg_attr(not(feature = "std"), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::B256;
use openzeppelin_crypto::{
    merkle::{self, Verifier},
    KeccakBuilder,
};
use stylus_sdk::{
    alloy_sol_types::sol,
    prelude::{entrypoint, public, storage},
    stylus_proc::SolidityError,
};

sol! {
    error MerkleProofInvalidMultiProofLength();
    error MerkleProofInvalidRootChild();
    error MerkleProofInvalidTotalHashes();
}

#[derive(SolidityError)]
pub enum VerifierError {
    InvalidProofLength(MerkleProofInvalidMultiProofLength),
    InvalidRootChild(MerkleProofInvalidRootChild),
    InvalidTotalHashes(MerkleProofInvalidTotalHashes),
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
        }
    }
}

#[entrypoint]
#[storage]
struct VerifierContract {}

#[public]
impl VerifierContract {
    pub fn verify(&self, proof: Vec<B256>, root: B256, leaf: B256) -> bool {
        let proof: Vec<[u8; 32]> = proof.into_iter().map(|m| *m).collect();
        Verifier::<KeccakBuilder>::verify(&proof, *root, *leaf)
    }

    pub fn verify_multi_proof(
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
