//! Projective curve points.
use super::{affine::AffinePoint, curve::PrimeCurveParams, field::Field};

/// Point on a Weierstrass curve in projective (homogeneous) coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ProjectivePoint<C: PrimeCurveParams> {
    /// X-coordinate.
    pub x: C::FieldElement,
    /// Y-coordinate.
    pub y: C::FieldElement,
    /// Z-coordinate.
    pub z: C::FieldElement,
}

impl<C: PrimeCurveParams> ProjectivePoint<C> {
    /// The base point of curve `C`.
    pub const GENERATOR: Self =
        Self { x: C::GENERATOR.0, y: C::GENERATOR.1, z: C::FieldElement::ONE };
    /// The "point at infinity".
    pub const IDENTITY: Self = Self {
        x: C::FieldElement::ZERO,
        y: C::FieldElement::ONE,
        z: C::FieldElement::ZERO,
    };

    /// Returns the affine representation of this point, or `None` if it is the
    /// identity.
    pub fn to_affine(&self) -> Option<AffinePoint<C>> {
        <C::FieldElement as Field>::invert(&self.z)
            .map(|zinv| AffinePoint { x: self.x * zinv, y: self.y * zinv })
    }
}

impl<C> PartialEq for ProjectivePoint<C>
where
    C: PrimeCurveParams,
{
    fn eq(&self, other: &Self) -> bool {
        // Since projective points are members of equivalence classes, we can't
        // just compare the point's components.
        //
        // Converting to affine space gives us a unique point.
        let lhs = self.to_affine();
        let rhs = other.to_affine();
        lhs == rhs
    }
}

impl<C> Default for ProjectivePoint<C>
where
    C: PrimeCurveParams,
{
    fn default() -> Self {
        Self::IDENTITY
    }
}
