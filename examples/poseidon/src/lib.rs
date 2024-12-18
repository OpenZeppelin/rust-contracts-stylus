#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::B256;
use openzeppelin_crypto::{
    field::instance::FpVesta,
    hash::Hasher,
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

        // Compute hash from bytes.
        let mut first_hasher = Poseidon2::<VestaParams, FpVesta>::new();
        first_hasher.update(&input);
        let hash = first_hasher.finalize();

        Ok(B256::from(hash))
    }
}
