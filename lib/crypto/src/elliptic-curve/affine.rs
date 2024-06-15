//! Affine curve points.
use super::curve::PrimeCurveParams;

/// Point on a Weierstrass curve in affine coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffinePoint<C: PrimeCurveParams> {
    /// x-coordinate.
    pub x: C::FieldElement,
    /// y-coordinate.
    pub y: C::FieldElement,
}

impl<C: PrimeCurveParams> AffinePoint<C> {
    /// The base point of curve `C`.
    pub const GENERATOR: Self = Self { x: C::GENERATOR.0, y: C::GENERATOR.1 };
}
