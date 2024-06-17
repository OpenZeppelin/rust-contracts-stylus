//! Affine curve points.
use core::ops::Neg;

use crate::elliptic_curve::curve::PrimeCurve;

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

impl<C> Neg for AffinePoint<C>
where
    C: PrimeCurve,
{
    type Output = Self;

    fn neg(self) -> Self {
        AffinePoint { x: self.x, y: -self.y }
    }
}

impl<C> Neg for &AffinePoint<C>
where
    C: PrimeCurve,
{
    type Output = AffinePoint<C>;

    fn neg(self) -> AffinePoint<C> {
        -(*self)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::AffinePoint;
    use crate::elliptic_curve::p256::P256;

    #[test]
    fn affine_negation() {
        let basepoint = AffinePoint::<P256>::GENERATOR;
        assert_eq!(-(-basepoint), basepoint);
    }
}
