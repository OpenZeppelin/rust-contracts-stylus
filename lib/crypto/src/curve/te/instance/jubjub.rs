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

const G_GENERATOR_X: Fq =
    fp_from_num!("5299619240641551281634865583518297030282874472190772894086521144482721001553");
const G_GENERATOR_Y: Fq =
    fp_from_num!("16950150798460657717958625567821834550301663161624707787222815936182638968203");

/// Base Field for [`JubjubConfig`].
pub type Fq = Fp256<JubjubFqParam>;

/// Base Field parameters for [`JubjubConfig`].
pub struct JubjubFqParam;

impl FpParams<LIMBS_256> for JubjubFqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("21888242871839275222246405745257275088548364400416034343698204186575808495617");
}

/// Scalar Field for [`JubjubConfig`].
pub type Fr = Fp256<JubjubFrParam>;

/// Scalar Field parameters for [`JubjubConfig`].
pub struct JubjubFrParam;

impl FpParams<LIMBS_256> for JubjubFrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("723700557733226221397318656304299424085711635937990760600195093828545425857");
}

/// Jubjub Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct JubjubConfig;

impl CurveConfig for JubjubConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("21711016731996786641919559689128982722488122124807605757398297001483711807481");
}

impl TECurveConfig for JubjubConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("1").ct_neg();
    const COEFF_D: Self::BaseField = fp_from_num!("168696");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for JubjubConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("40962");
    const COEFF_B: Self::BaseField = fp_from_num!("1");
}
