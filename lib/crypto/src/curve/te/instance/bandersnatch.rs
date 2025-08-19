//! [Bandersnatch Elliptic Curve] parameters.
//!
//! [Bandersnatch Elliptic Curve]: <https://eprint.iacr.org/2021/1152>

use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_hex, fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_hex!(
    "29c132cc2c0b34c5743711777bbe42f32b79c022ad998465e1e71866a252ae18"
);
const G_GENERATOR_Y: Fq = fp_from_hex!(
    "2a6c669eda123e0f157d8b50badcd586358cad81eee464605e3167b6cc974166"
);

/// Base Field for [`BandersnatchConfig`].
pub type Fq = Fp256<BandersnatchFqParam>;
/// Base Field parameters for [`BandersnatchConfig`].
pub struct BandersnatchFqParam;

impl FpParams<LIMBS_256> for BandersnatchFqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("52435875175126190479447740508185965837690552500527637822603658699938581184513");
}

/// Scalar Field for [`BandersnatchConfig`].
pub type Fr = Fp256<BandersnatchFrParam>;
/// Scalar Field parameters for [`BandersnatchConfig`].
pub struct BandersnatchFrParam;

impl FpParams<LIMBS_256> for BandersnatchFrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("13108968793781547619861935127046491459309155893440570251786403306729687672801");
}

/// Bandersnatch Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BandersnatchConfig;

impl CurveConfig for BandersnatchConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[4];
    const COFACTOR_INV: Fr = fp_from_num!("9831726595336160714896451345284868594481866920080427688839802480047265754601");
}

impl TECurveConfig for BandersnatchConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("5").ct_neg();
    const COEFF_D: Self::BaseField =
        fp_from_num!("45022363124591815672509500913686876175488063829319466900776701791074614335719");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for BandersnatchConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_hex!(
        "4247698f4e32ad45a293959b4ca17afa4a2d2317e4c6ce5023e1fd63d1b5de98"
    );
    const COEFF_B: Self::BaseField = fp_from_hex!(
        "300c3385d13bedb7c9e229e185c4ce8b1dd3b71366bb97c30855c0aa41d62727"
    );
}
