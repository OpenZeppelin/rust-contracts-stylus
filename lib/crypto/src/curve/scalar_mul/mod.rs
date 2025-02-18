use num_traits::Zero;

use crate::{
    curve::short_weierstrass::{Affine, Projective, SWCurveConfig},
    field::group::AdditiveGroup,
};
// TODO#q: move scalar_mul to curve module

/// Standard double-and-add method for multiplication by a scalar.
#[inline(always)]
pub fn sw_double_and_add_affine<P: SWCurveConfig>(
    base: &Affine<P>,
    scalar: impl AsRef<[u64]>,
) -> Projective<P> {
    let mut res = Projective::zero();
    for b in ark_ff::BitIteratorBE::without_leading_zeros(scalar) {
        res.double_in_place();
        if b {
            res += base
        }
    }

    res
}

/// Standard double-and-add method for multiplication by a scalar.
#[inline(always)]
pub fn sw_double_and_add_projective<P: SWCurveConfig>(
    base: &Projective<P>,
    scalar: impl AsRef<[u64]>,
) -> Projective<P> {
    let mut res = Projective::zero();
    for b in ark_ff::BitIteratorBE::without_leading_zeros(scalar) {
        res.double_in_place();
        if b {
            res += base
        }
    }

    res
}
