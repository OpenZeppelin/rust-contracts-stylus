#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U256;
use openzeppelin_crypto::{
    arithmetic::{uint::Uint, BigInteger},
    field::{instance::FpBN256, prime::PrimeField},
    poseidon2::{instance::bn256::BN256Params, Poseidon2},
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct PoseidonExample;

#[public]
impl PoseidonExample {
    fn hash(&mut self, inputs: [U256; 2]) -> U256 {
        let mut hasher = Poseidon2::<BN256Params, FpBN256>::new();

        for input in inputs.iter() {
            let fp = FpBN256::from_bigint(Uint::from_bytes_le(
                &input.to_le_bytes_vec(),
            ));
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        let hash = hash.into_bigint().into_bytes_le();

        U256::from_le_slice(&hash)
    }
}
