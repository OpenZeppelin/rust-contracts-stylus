//! This module contains the [secp256k1] curve configuration.
//!
//! [secp256k1]: <https://www.secg.org/sec2-v2.pdf>
use crate::{
    arithmetic::uint::U256,
    curve::{
        sw::{Affine, SWCurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq =
        fp_from_num!("55066263022277343669578718895168534326250603453777594175500187360389116729240");

const G_GENERATOR_Y: Fq =
        fp_from_num!("32670510020758816978083085130507043184471273380659243275938904335757337482424");

/// Base Field for [`Secp256k1Config`].
pub type Fq = Fp256<Secp256k1FqParam>;
/// Base Field parameters for [`Secp256k1Config`].
pub struct Secp256k1FqParam;

impl FpParams<LIMBS_256> for Secp256k1FqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("3");
    const MODULUS: U256 = from_num!("115792089237316195423570985008687907853269984665640564039457584007908834671663");
}

/// Scalar Field for [`Secp256k1Config`].
pub type Fr = Fp256<Secp256k1FrParam>;
/// Scalar Field parameters for [`Secp256   k1Config`].
pub struct Secp256k1FrParam;

impl FpParams<LIMBS_256> for Secp256k1FrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("7");
    const MODULUS: U256 = from_num!("115792089237316195423570985008687907852837564279074904382605163141518161494337");
}

/// Secp256k1's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Secp256k1Config;

impl CurveConfig for Secp256k1Config {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[0x1, 0x0];
    const COFACTOR_INV: Fr = Fr::ONE;
}

impl SWCurveConfig for Secp256k1Config {
    const COEFF_A: Fq = Fq::ZERO;
    const COEFF_B: Fq = fp_from_num!("7");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}
