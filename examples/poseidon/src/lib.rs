#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::B256;
use openzeppelin_crypto::{
    bigint::BigInteger,
    field::{instance::FpVesta, prime::PrimeField},
    poseidon2::{instance::vesta::VestaParams, Poseidon2},
};
use stylus_sdk::prelude::{entrypoint, public, storage};

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

        let mut hasher = Poseidon2::<VestaParams, FpVesta>::new();

        for i in 0..1 {
            let fp = FpVesta::from(i);
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        let hash = hash.into_bigint().into_bytes_le();

        Ok(B256::from_slice(&hash))
    }
}
