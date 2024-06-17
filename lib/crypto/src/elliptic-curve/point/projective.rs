//! Projective curve points.
use core::{
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use bigint::Bounded;

use crate::elliptic_curve::{
    curve::PrimeCurve, field::Field, point::arithmetic::PointArithmetic,
};

use super::affine::AffinePoint;

/// Point on a Weierstrass curve in projective (homogeneous) coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ProjectivePoint<C: PrimeCurve> {
    /// X-coordinate.
    pub x: C::FieldElement,
    /// Y-coordinate.
    pub y: C::FieldElement,
    /// Z-coordinate.
    pub z: C::FieldElement,
}

impl<C: PrimeCurve> ProjectivePoint<C> {
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

    /// Returns `-self`.
    #[must_use]
    pub fn neg(&self) -> Self {
        Self { x: self.x, y: -self.y, z: self.z }
    }

    /// Returns `self + other`.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        C::PointArithmetic::add(self, other)
    }

    /// Returns `self + other`.
    #[must_use]
    fn add_mixed(&self, other: &AffinePoint<C>) -> Self {
        C::PointArithmetic::add_mixed(self, other)
    }

    /// Returns `self - other`.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }

    /// Returns `scalar * self`.
    fn mul(&self, scalar: C::Uint) -> Self {
        let one = C::Uint::from(1u64);
        let mut result = ProjectivePoint::IDENTITY;
        let mut addend = *self;
        for shift in 0..C::Uint::BITS {
            let bit = (scalar >> shift) & one;
            if bit == one {
                result += addend;
            }
            addend = C::PointArithmetic::double(&addend);
        }

        result
    }
}

impl<C> PartialEq for ProjectivePoint<C>
where
    C: PrimeCurve,
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
    C: PrimeCurve,
{
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl<C> Add<ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn add(self, other: ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::add(&self, &other)
    }
}

impl<C> Add<&ProjectivePoint<C>> for &ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn add(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::add(self, other)
    }
}

impl<C> Add<&ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn add(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::add(&self, other)
    }
}

impl<C> AddAssign<ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn add_assign(&mut self, rhs: ProjectivePoint<C>) {
        *self = ProjectivePoint::add(self, &rhs);
    }
}

impl<C> AddAssign<&ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn add_assign(&mut self, rhs: &ProjectivePoint<C>) {
        *self = ProjectivePoint::add(self, rhs);
    }
}

impl<C> Add<AffinePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn add(self, other: AffinePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::add_mixed(&self, &other)
    }
}

impl<C> Add<&AffinePoint<C>> for &ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn add(self, other: &AffinePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::add_mixed(self, other)
    }
}

impl<C> Add<&AffinePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn add(self, other: &AffinePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::add_mixed(&self, other)
    }
}

impl<C> AddAssign<AffinePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn add_assign(&mut self, rhs: AffinePoint<C>) {
        *self = ProjectivePoint::add_mixed(self, &rhs);
    }
}

impl<C> AddAssign<&AffinePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn add_assign(&mut self, rhs: &AffinePoint<C>) {
        *self = ProjectivePoint::add_mixed(self, rhs);
    }
}

impl<C> Sum for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ProjectivePoint::IDENTITY, |a, b| a + b)
    }
}

impl<'a, C> Sum<&'a ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn sum<I: Iterator<Item = &'a ProjectivePoint<C>>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl<C> Sub<ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn sub(self, other: ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::sub(&self, &other)
    }
}

impl<C> Sub<&ProjectivePoint<C>> for &ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn sub(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::sub(self, other)
    }
}

impl<C> Sub<&ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn sub(self, other: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        ProjectivePoint::sub(&self, other)
    }
}

impl<C> SubAssign<ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn sub_assign(&mut self, rhs: ProjectivePoint<C>) {
        *self = ProjectivePoint::sub(self, &rhs);
    }
}

impl<C> SubAssign<&ProjectivePoint<C>> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn sub_assign(&mut self, rhs: &ProjectivePoint<C>) {
        *self = ProjectivePoint::sub(self, rhs);
    }
}

impl<C> Mul<C::Uint> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = Self;

    fn mul(self, scalar: C::Uint) -> Self {
        ProjectivePoint::mul(&self, scalar)
    }
}

impl<C> Mul<&C::Uint> for &ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn mul(self, scalar: &C::Uint) -> ProjectivePoint<C> {
        ProjectivePoint::mul(self, *scalar)
    }
}

impl<C> MulAssign<C::Uint> for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    fn mul_assign(&mut self, scalar: C::Uint) {
        *self = ProjectivePoint::mul(self, scalar);
    }
}

impl<C> Neg for ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn neg(self) -> ProjectivePoint<C> {
        ProjectivePoint::neg(&self)
    }
}

impl<'a, C> Neg for &'a ProjectivePoint<C>
where
    C: PrimeCurve,
{
    type Output = ProjectivePoint<C>;

    fn neg(self) -> ProjectivePoint<C> {
        ProjectivePoint::neg(self)
    }
}
