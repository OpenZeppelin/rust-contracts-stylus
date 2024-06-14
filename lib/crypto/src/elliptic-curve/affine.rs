//! Affine curve points.
use core::marker::PhantomData;

use bn::U256;

use super::curve::PrimeCurveParams;

/// Point on a Weierstrass curve in affine coordinates.
#[derive(Clone, Copy, Debug)]
pub struct AffinePoint<C: PrimeCurveParams> {
    /// x-coordinate.
    pub x: U256,
    /// y-coordinate.
    pub y: U256,

    marker: PhantomData<C>,
}

impl<C: PrimeCurveParams> AffinePoint<C> {
    pub const GENERATOR: Self =
        Self { x: C::GENERATOR.0, y: C::GENERATOR.1, marker: PhantomData };
}
