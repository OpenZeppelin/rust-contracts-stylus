#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::B256;
use ark_ec::Group;
use ark_ff::{BigInteger, Fp256, MontBackend, MontConfig, PrimeField};
use renegade_crypto::hash::Poseidon2Sponge;
use stylus_sdk::prelude::{entrypoint, public, storage};

pub type SystemCurveGroup = ark_bn254::G1Projective;
pub type ScalarField = <SystemCurveGroup as Group>::ScalarField;

// #[derive(MontConfig)]
// #[modulus =
// "21888242871839275222246405745257275088548364400416034343698204186575808495617"
// ] #[generator = "7"]
// pub struct FqConfig;
// pub type FpBN256 = Fp256<MontBackend<FqConfig, 4>>;

#[entrypoint]
#[storage]
struct PoseidonExample {}

#[public]
impl PoseidonExample {
    pub fn hash(
        &mut self,
        data: stylus_sdk::abi::Bytes,
    ) -> Result<B256, Vec<u8>> {
        let input = data.to_vec();

        let mut hasher = Poseidon2Sponge::new();

        for i in 0..2 {
            let fp = ScalarField::from(i);
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        let hash = hash.into_bigint().to_bytes_le();

        Ok(B256::from_slice(&hash))
    }
}
