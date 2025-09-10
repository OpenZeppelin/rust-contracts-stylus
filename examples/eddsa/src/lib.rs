#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::needless_pass_by_value, clippy::unused_self)]

extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_crypto::{
    curve::te::instance::curve25519::Curve25519FqParam,
    eddsa::{AffinePoint, Scalar, Signature, VerifyingKey},
    field::fp::Fp256,
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct EddsaExample;

#[public]
impl EddsaExample {
    /// Verification api is slightly different from canonical implementation
    /// missing compressed points.
    fn verify(
        &self,
        verifying_key: [alloy_primitives::U256; 2],
        signature: [alloy_primitives::U256; 3],
        message: Bytes,
    ) -> bool {
        let verifying_key = VerifyingKey::from_affine(AffinePoint {
            x: Fp256::<Curve25519FqParam>::from_bigint(verifying_key[0].into()),
            y: Fp256::<Curve25519FqParam>::from_bigint(verifying_key[1].into()),
        });

        let signature = Signature::from_affine_R_s(
            AffinePoint {
                x: Fp256::<Curve25519FqParam>::from_bigint(signature[0].into()),
                y: Fp256::<Curve25519FqParam>::from_bigint(signature[1].into()),
            },
            Scalar::from_bigint(signature[2].into()),
        );

        verifying_key.is_valid(&message, &signature)
    }
}
