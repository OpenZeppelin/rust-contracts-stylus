#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U256;
use ark_ff::{BigInteger, PrimeField};
use renegade_crypto::hash::{Poseidon2Sponge, ScalarField};
use stylus_sdk::prelude::{entrypoint, public, storage};

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
    pub fn hash(&mut self, inputs: [U256; 2]) -> Result<U256, Vec<u8>> {
        let mut hasher = Poseidon2Sponge::new();

        for input in inputs.iter() {
            let fp =
                ScalarField::from_le_bytes_mod_order(&input.to_le_bytes_vec());
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        let hash = hash.into_bigint().to_bytes_le();

        Ok(U256::from_le_slice(&hash))
    }
}
