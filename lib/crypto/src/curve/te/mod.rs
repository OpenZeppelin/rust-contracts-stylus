//! This module contains definitions for the [Twisted Edwards model] of the
//! curve.
//!
//! [Twisted Edwards model]: https://www.hyperelliptic.org/EFD/g1p/auto-twisted.html
use num_traits::Zero;

mod affine;
pub use affine::*;

mod projective;
pub use projective::*;

use crate::{
    bits::BitIteratorBE,
    curve::AffineRepr,
    field::{group::AdditiveGroup, prime::PrimeField},
};

/// Constants and convenience functions
/// that define the [Twisted Edwards model] of the curve.
///
/// In this model, the curve equation is `a * x² + y² = 1 + d * x² * y²`, for
/// constants `a` and `d`.
///
/// [Twisted Edwards model]: https://www.hyperelliptic.org/EFD/g1p/auto-twisted.html
pub trait TECurveConfig: super::CurveConfig {
    /// Coefficient `a` of the curve equation.
    const COEFF_A: Self::BaseField;
    /// Coefficient `d` of the curve equation.
    const COEFF_D: Self::BaseField;
    /// Generator of the prime-order subgroup.
    const GENERATOR: Affine<Self>;

    /// Model parameters for the Montgomery curve that is birationally
    /// equivalent to this curve.
    type MontCurveConfig: MontCurveConfig<BaseField = Self::BaseField>;

    /// Helper method for computing `elem * Self::COEFF_A`.
    ///
    /// The default implementation should be overridden only if
    /// the product can be computed faster than standard field multiplication
    /// (eg: via doubling if `COEFF_A == 2`, or if `COEFF_A.is_zero()`).
    #[inline(always)]
    fn mul_by_a(elem: Self::BaseField) -> Self::BaseField {
        elem * Self::COEFF_A
    }

    /// Checks that the current point is in the prime order subgroup given
    /// the point on the curve.
    fn is_in_correct_subgroup_assuming_on_curve(item: &Affine<Self>) -> bool {
        Self::mul_affine(item, Self::ScalarField::characteristic()).is_zero()
    }

    /// Performs cofactor clearing.
    /// The default method is simply to multiply by the cofactor.
    /// For some curve families though, it is sufficient to multiply
    /// by a smaller scalar.
    fn clear_cofactor(item: &Affine<Self>) -> Affine<Self> {
        item.mul_by_cofactor()
    }

    /// Default implementation of group multiplication for projective
    /// coordinates
    fn mul_projective(
        base: &Projective<Self>,
        scalar: impl BitIteratorBE,
    ) -> Projective<Self> {
        let mut res = Projective::zero();
        for b in scalar.bit_be_trimmed_iter() {
            res.double_in_place();
            if b {
                res += base;
            }
        }

        res
    }

    /// Default implementation of group multiplication for affine
    /// coordinates
    fn mul_affine(
        base: &Affine<Self>,
        scalar: impl BitIteratorBE,
    ) -> Projective<Self> {
        let mut res = Projective::zero();
        for b in scalar.bit_be_trimmed_iter() {
            res.double_in_place();
            if b {
                res += base;
            }
        }

        res
    }
}

/// Constants and convenience functions that collectively define the [Montgomery model](https://www.hyperelliptic.org/EFD/g1p/auto-montgom.html)
/// of the curve.
///
/// In this model, the curve equation is `b * y² = x³ + a * x² + x`, for
/// constants `a` and `b`.
pub trait MontCurveConfig: super::CurveConfig {
    /// Coefficient `a` of the curve equation.
    const COEFF_A: Self::BaseField;
    /// Coefficient `b` of the curve equation.
    const COEFF_B: Self::BaseField;

    /// Model parameters for the Twisted Edwards curve that is birationally
    /// equivalent to this curve.
    type TECurveConfig: TECurveConfig<BaseField = Self::BaseField>;
}

#[cfg(test)]
mod test {
    use num_traits::Zero;

    use crate::{
        arithmetic::uint::U256,
        curve::{
            te::{Affine, MontCurveConfig, TECurveConfig},
            AffineRepr, CurveConfig, CurveGroup,
        },
        field::fp::{Fp256, FpParams, LIMBS_256},
        fp_from_num, from_num,
    };

    #[derive(Clone, Default, PartialEq, Eq)]
    struct Config;

    type Fq = Fp256<FqParam>;
    struct FqParam;
    impl FpParams<LIMBS_256> for FqParam {
        const GENERATOR: Fp256<Self> = fp_from_num!("2");
        const MODULUS: U256 = from_num!("57896044618658097711785492504343953926634992332820282019728792003956564819949");
    }

    type Fr = Fp256<FrParam>;
    struct FrParam;
    impl FpParams<LIMBS_256> for FrParam {
        const GENERATOR: Fp256<Self> = fp_from_num!("2");
        const MODULUS: U256 = from_num!("7237005577332262213973186563042994240857116359379907606001950938285454250989");
    }

    impl CurveConfig for Config {
        type BaseField = Fq;
        type ScalarField = Fr;

        const COFACTOR: &'static [u64] = &[8];
        const COFACTOR_INV: Fr = fp_from_num!("2713877091499598330239944961141122840321418634767465352250731601857045344121");
    }

    impl TECurveConfig for Config {
        type MontCurveConfig = Self;

        const COEFF_A: Self::BaseField = fp_from_num!("-1");
        const COEFF_D: Self::BaseField = fp_from_num!("37095705934669439343138083508754565189542113879843219016388785533085940283555");
        const GENERATOR: Affine<Self> =
            Affine::new_unchecked(GENERATOR_X, GENERATOR_Y);
    }

    impl MontCurveConfig for Config {
        type TECurveConfig = Self;

        const COEFF_A: Self::BaseField = fp_from_num!("486662");
        const COEFF_B: Self::BaseField = fp_from_num!("57896044618658097711785492504343953926634992332820282019728792003956564333285");
    }

    /// GENERATOR_X =
    /// 15112221349535400772501151409588531511454012693041857206046113283949847762202
    const GENERATOR_X: Fq =
        fp_from_num!("15112221349535400772501151409588531511454012693041857206046113283949847762202");

    /// GENERATOR_Y =
    /// (4/5)
    /// 46316835694926478169428394003475163141307993866256225615783033603165251855960
    const GENERATOR_Y: Fq =
        fp_from_num!("46316835694926478169428394003475163141307993866256225615783033603165251855960");

    #[test]
    fn scalar_mul() {
        assert!(Affine::<Config>::generator()
            .mul_bigint(0u32)
            .into_affine()
            .is_zero());
    }
}
