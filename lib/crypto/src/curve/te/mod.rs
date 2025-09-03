//! This module contains definitions for the [Twisted Edwards model] of the
//! curve.
//!
//! [Twisted Edwards model]: https://www.hyperelliptic.org/EFD/g1p/auto-twisted.html
use num_traits::Zero;

mod affine;
pub use affine::*;

mod projective;
pub use projective::*;

pub mod instance;

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

    /// Checks that the current point is in the prime order subgroup, assuming
    /// the point is already on the curve.
    fn is_in_prime_order_subgroup(item: &Affine<Self>) -> bool {
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
    /// coordinates.
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
    /// coordinates.
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
