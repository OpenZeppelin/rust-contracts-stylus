//! This module contains the [Jubjub Elliptic Curve] configuration.
//!
//! [Jubjub Elliptic Curve]: <https://zips.z.cash/protocol/protocol.pdf>

use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_num!("8076246640662884909881801758704306714034609987455869804520522091855516602923");
const G_GENERATOR_Y: Fq = fp_from_num!("13262374693698910701929044844600465831413122818447359594527400194675274060458");

/// Base Field for [`JubjubConfig`].
pub type Fq = Fp256<JubjubFqParam>;
/// Base Field parameters for [`JubjubConfig`].
pub struct JubjubFqParam;

impl FpParams<LIMBS_256> for JubjubFqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("52435875175126190479447740508185965837690552500527637822603658699938581184513");
}

/// Scalar Field for [`JubjubConfig`].
pub type Fr = Fp256<JubjubFrParam>;
/// Scalar Field parameters for [`JubjubConfig`].
pub struct JubjubFrParam;

impl FpParams<LIMBS_256> for JubjubFrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("6554484396890773809930967563523245729705921265872317281365359162392183254199");
}

/// Jubjub Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct JubjubConfig;

impl CurveConfig for JubjubConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("819310549611346726241370945440405716213240158234039660170669895299022906775");
}

impl TECurveConfig for JubjubConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("1").ct_neg();
    const COEFF_D: Self::BaseField = fp_from_num!("19257038036680949359750312669786877991949435402254120286184196891950884077233");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for JubjubConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("40962");
    const COEFF_B: Self::BaseField = fp_from_num!("1");
}
