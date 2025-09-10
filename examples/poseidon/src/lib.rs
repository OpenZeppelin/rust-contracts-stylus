#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::unused_self)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_crypto::{
    arithmetic::uint::U256,
    field::instance::FpBN256,
    poseidon2::{instance::bn256::BN256Params, Poseidon2},
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct PoseidonExample;

#[public]
impl PoseidonExample {
    fn hash(
        &self,
        inputs: [alloy_primitives::U256; 2],
    ) -> alloy_primitives::U256 {
        let mut hasher = Poseidon2::<BN256Params, FpBN256>::new();

        for input in &inputs {
            let fp = FpBN256::from_bigint(U256::from(*input));
            hasher.absorb(&fp);
        }

        let hash = hasher.squeeze();
        hash.into_bigint().into()
    }
}
