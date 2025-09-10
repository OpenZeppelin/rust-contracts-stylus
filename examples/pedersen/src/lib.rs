#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::unused_self)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_crypto::{
    arithmetic::uint::U256,
    curve::sw::instance::starknet::StarknetCurveConfig,
    pedersen::{instance::starknet::StarknetPedersenParams, Pedersen},
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct PedersenExample;

#[public]
impl PedersenExample {
    fn hash(
        &self,
        inputs: [alloy_primitives::U256; 2],
    ) -> alloy_primitives::U256 {
        let hasher =
            Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();

        let inputs: Vec<U256> = inputs.iter().map(|x| U256::from(*x)).collect();

        let hash = hasher.hash(inputs[0], inputs[1]);
        hash.expect("Failed to hash").into_bigint().into()
    }
}
