//! [Baby Jubjub Elliptic Curve] parameters.
//!
//! [Baby Jubjub Elliptic Curve]: https://eips.ethereum.org/EIPS/eip-2494

use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_num!("995203441582195749578291179787384436505546430278305826713579947235728471134");

const G_GENERATOR_Y: Fq = fp_from_num!("5472060717959818805561601436314318772137091100104008585924551046643952123905");

/// Base Field for [`BabyJubjubConfig`].
pub type Fq = Fp256<BabyJubjubFqParam>;
/// Base Field parameters for [`BabyJubjubConfig`].
pub struct BabyJubjubFqParam;

impl FpParams<LIMBS_256> for BabyJubjubFqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("21888242871839275222246405745257275088548364400416034343698204186575808495617");
}

/// Scalar Field for [`BabyJubjubConfig`].
pub type Fr = Fp256<BabyJubjubFrParam>;
/// Scalar Field parameters for [`BabyJubjubConfig`].
pub struct BabyJubjubFrParam;

impl FpParams<LIMBS_256> for BabyJubjubFrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("2736030358979909402780800718157159386076813972158567259200215660948447373041");
}

/// Baby Jubjub's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BabyJubjubConfig;

impl CurveConfig for BabyJubjubConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("2394026564107420727433200628387514462817212225638746351800188703329891451411");
}

impl TECurveConfig for BabyJubjubConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("168700");
    const COEFF_D: Self::BaseField = fp_from_num!("168696");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for BabyJubjubConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("168698");
    const COEFF_B: Self::BaseField = fp_from_num!("1");
}
