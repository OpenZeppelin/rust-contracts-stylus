//! Affine curve points.
use super::curve::PrimeCurve;

/// Point on a Weierstrass curve in affine coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffinePoint<C: PrimeCurve> {
    /// x-coordinate.
    pub x: C::FieldElement,
    /// y-coordinate.
    pub y: C::FieldElement,
}

impl<C: PrimeCurve> AffinePoint<C> {
    /// The base point of curve `C`.
    pub const GENERATOR: Self = Self { x: C::GENERATOR.0, y: C::GENERATOR.1 };
}
