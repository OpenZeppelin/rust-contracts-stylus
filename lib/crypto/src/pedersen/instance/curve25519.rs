//! TODO.
use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_hex, from_num,
    pedersen::params::PedersenParams,
};
#[derive(Clone, Default, PartialEq, Eq)]
/// Curve25519's Curve Details.
pub struct Curve25519Config;

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

impl CurveConfig for Curve25519Config {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("2713877091499598330239944961141122840321418634767465352250731601857045344121");
}

const GENERATOR_X: Fq =
        fp_from_num!("15112221349535400772501151409588531511454012693041857206046113283949847762202");

const GENERATOR_Y: Fq =
        fp_from_num!("46316835694926478169428394003475163141307993866256225615783033603165251855960");

impl TECurveConfig for Curve25519Config {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("1").ct_neg();
    const COEFF_D: Self::BaseField = fp_from_num!("37095705934669439343138083508754565189542113879843219016388785533085940283555");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(GENERATOR_X, GENERATOR_Y);
}

impl MontCurveConfig for Curve25519Config {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("486662");
    const COEFF_B: Self::BaseField = fp_from_num!("57896044618658097711785492504343953926634992332820282019728792003956564333285");
}

#[derive(Clone, Default, PartialEq, Eq)]
/// Pedersen Hash parameters for Curve25519.
pub struct Curve25519PedersenParams;

impl PedersenParams<Curve25519Config> for Curve25519PedersenParams {
    type AffineRepr = Affine<Curve25519Config>;

    /// Low part bits.
    const LOW_PART_BITS: u32 = 248;
    /// Low part mask. (2**248 - 1)
    const LOW_PART_MASK: U256 = from_hex!(
        "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    const N_ELEMENT_BITS_HASH: usize = 252;
    // Note: These are placeholder values for Curve25519 Pedersen hash
    // parameters. In practice, Curve25519 is not typically used for
    // Pedersen hashing. These values would need to be replaced with actual
    // computed parameters for a real Curve25519 Pedersen hash
    // implementation.
    const P_0: Affine<Curve25519Config> =
        Affine::new_unchecked(fp_from_num!("0"), fp_from_num!("1"));
    const P_1: Affine<Curve25519Config> =
        Affine::new_unchecked(fp_from_num!("0"), fp_from_num!("1"));
    const P_2: Affine<Curve25519Config> =
        Affine::new_unchecked(fp_from_num!("0"), fp_from_num!("1"));
    const P_3: Affine<Curve25519Config> =
        Affine::new_unchecked(fp_from_num!("0"), fp_from_num!("1"));
    const P_4: Affine<Curve25519Config> =
        Affine::new_unchecked(fp_from_num!("0"), fp_from_num!("1"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pedersen::Pedersen;

    #[test]
    fn test_curve25519_pedersen_params_compiles() {
        // This test verifies that the Curve25519 PedersenParams implementation
        // compiles and can be used with the Pedersen hash struct.
        let _pedersen =
            Pedersen::<Curve25519PedersenParams, Curve25519Config>::new();

        // Verify that the constants are accessible
        assert_eq!(Curve25519PedersenParams::N_ELEMENT_BITS_HASH, 252);
        assert_eq!(Curve25519PedersenParams::LOW_PART_BITS, 248);
    }
}
