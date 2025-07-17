//! [Bandersnatch Elliptic Curve] parameters.
//!
//! [Bandersnatch Elliptic Curve]: https://eprint.iacr.org/2021/1152

use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_num!("19232933407424889111104940496320988988172100217998297030544530116054238325480");
const G_GENERATOR_Y: Fq = fp_from_num!("17060630597514316424753713474449400526842970574619404045211133826199305117375");

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
    const COFACTOR_INV: Fr = fp_from_num!("9820571595336158597396451345284868594481866920080427688839802480047265754601");
}

impl TECurveConfig for BandersnatchConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("5").ct_neg();
    const COEFF_D: Self::BaseField = fp_from_num!("138827208126141220649022263972958607803171449701953573178309673572579671231137");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for BandersnatchConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("40962");
    const COEFF_B: Self::BaseField = fp_from_num!("1");
}
