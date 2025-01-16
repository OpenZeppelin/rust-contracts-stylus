#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U256;
use openzeppelin_crypto::{
    arithmetic::{uint::Uint, BigInteger},
    field::{
        group::AdditiveGroup, instance::FpBN256, prime::PrimeField, Field,
    },
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[entrypoint]
#[storage]
struct MathExample {}

#[public]
impl MathExample {
    pub fn compute(&mut self, inputs: [U256; 2]) -> Result<U256, Vec<u8>> {
        let inputs: Vec<_> = inputs
            .iter()
            .map(|input| {
                FpBN256::from_bigint(Uint::from_bytes_le(
                    &input.to_le_bytes_vec(),
                ))
            })
            .collect();

        let mut res = FpBN256::ONE;
        for _ in 0..1000 {
            for input in inputs.iter() {
                res += input;
                // res.inverse();
                // res += input;
                // res.inverse();
                // res.double_in_place();
                // res.inverse();
                // res.double_in_place();
                // res.inverse();
            }
        }

        let res = res.into_bigint().into_bytes_le();

        Ok(U256::from_le_slice(&res))
    }
}
