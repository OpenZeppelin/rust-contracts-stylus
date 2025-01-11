#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U256;
use ark_ff::{
    AdditiveGroup, BigInteger, Field, Fp256, MontBackend, MontConfig,
    PrimeField,
};
use stylus_sdk::prelude::{entrypoint, public, storage};

#[derive(MontConfig)]
#[modulus = "21888242871839275222246405745257275088548364400416034343698204186575808495617"]
#[generator = "7"]
pub struct FqConfig;
/// Bn245 field
pub type ScalarField = Fp256<MontBackend<FqConfig, 4>>;

/*
pub struct FqConfig;

const _: () = {
    use ark_ff::{
        biginteger::arithmetic as fa,
        fields::{Fp, *},
        BigInt, BigInteger,
    };
    type B = BigInt<4usize>;
    type F = Fp<MontBackend<FqConfig, 4usize>, 4usize>;
    #[automatically_derived]
    impl MontConfig<4usize> for FqConfig {
        const GENERATOR: F = ark_ff::MontFp!("7");
        const MODULUS: B = BigInt([
            4891460686036598785u64,
            2896914383306846353u64,
            13281191951274694749u64,
            3486998266802970665u64,
        ]);
        const TWO_ADIC_ROOT_OF_UNITY: F = ark_ff::MontFp!("1748695177688661943023146337482803886740723238769601073607632802312037301404" );
    }
};
*/

pub type FpBN256 = Fp256<MontBackend<FqConfig, 4>>;

#[entrypoint]
#[storage]
struct MathExample {}

#[public]
impl MathExample {
    pub fn compute(&mut self, inputs: [U256; 2]) -> Result<U256, Vec<u8>> {
        let inputs: Vec<_> = inputs
            .iter()
            .map(|input| {
                FpBN256::from_le_bytes_mod_order(&input.to_le_bytes_vec())
            })
            .collect();

        let mut res = FpBN256::ONE;
        for _ in 0..1000 {
            for input in inputs.iter() {
                res += input;
                res.square_in_place();
            }
        }

        let res = res.into_bigint().to_bytes_le();

        Ok(U256::from_le_slice(&res))
    }
}
