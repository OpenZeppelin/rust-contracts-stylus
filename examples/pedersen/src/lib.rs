#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_crypto::{
    arithmetic::{uint::U256, BigInteger},
    field::prime::PrimeField,
    pedersen::{
        instance::starknet::{StarknetCurveConfig, StarknetPedersenParams},
        Pedersen,
    },
};
use stylus_sdk::prelude::*;
#[entrypoint]
#[storage]
struct PedersenExample;

#[public]
impl PedersenExample {
    fn hash(
        &mut self,
        inputs: [alloy_primitives::U256; 2],
    ) -> alloy_primitives::U256 {
        let hasher =
            Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();

        let inputs: Vec<U256> = inputs
            .iter()
            .map(|x| U256::from_bytes_le(&x.to_le_bytes_vec()))
            .collect();

        let hash = hasher.hash(inputs[0], inputs[1]);
        let hash = hash.expect("Failed to hash").into_bigint().into_bytes_le();

        alloy_primitives::U256::from_le_slice(&hash)
    }
}
