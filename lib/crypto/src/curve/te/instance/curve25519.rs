//! This module contains the [Curve25519] configuration.
//!
//! [Curve25519]: <https://www.rfc-editor.org/rfc/rfc7748>
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
        fp_from_num!("15112221349535400772501151409588531511454012693041857206046113283949847762202");

const G_GENERATOR_Y: Fq =
        fp_from_num!("46316835694926478169428394003475163141307993866256225615783033603165251855960");

/// Base Field for [`Curve25519Config`].
pub type Fq = Fp256<Curve25519FqParam>;
/// Base Field parameters for [`Curve25519Config`].
pub struct Curve25519FqParam;

impl FpParams<LIMBS_256> for Curve25519FqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("57896044618658097711785492504343953926634992332820282019728792003956564819949");
}

/// Scalar Field for [`Curve25519Config`].
pub type Fr = Fp256<Curve25519FrParam>;
/// Scalar Field parameters for [`Curve25519Config`].
pub struct Curve25519FrParam;

impl FpParams<LIMBS_256> for Curve25519FrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("7237005577332262213973186563042994240857116359379907606001950938285454250989");
}

/// Curve25519's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Curve25519Config;

impl CurveConfig for Curve25519Config {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("2713877091499598330239944961141122840321418634767465352250731601857045344121");
}

impl TECurveConfig for Curve25519Config {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("1").ct_neg();
    const COEFF_D: Self::BaseField = fp_from_num!("37095705934669439343138083508754565189542113879843219016388785533085940283555");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for Curve25519Config {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("486662");
    const COEFF_B: Self::BaseField = fp_from_num!("57896044618658097711785492504343953926634992332820282019728792003956564333285");
}
