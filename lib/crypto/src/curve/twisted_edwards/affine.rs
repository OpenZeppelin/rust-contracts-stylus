use core::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter},
    ops::{Add, Mul, Neg, Sub},
};

use educe::Educe;
use num_traits::{One, Zero};
use zeroize::Zeroize;

use super::{Projective, TECurveConfig};
use crate::{
    bits::BitIteratorBE,
    curve::AffineRepr,
    field::{group::AdditiveGroup, prime::PrimeField, Field},
};

/// Affine coordinates for a point on a twisted Edwards curve, over the
/// base field `P::BaseField`.
#[derive(Educe)]
#[educe(Copy, Clone, PartialEq, Eq, Hash)]
#[must_use]
pub struct Affine<P: TECurveConfig> {
    /// X coordinate of the point represented as a field element
    pub x: P::BaseField,
    /// Y coordinate of the point represented as a field element
    pub y: P::BaseField,
}

impl<P: TECurveConfig> Display for Affine<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.is_zero() {
            true => write!(f, "infinity"),
            false => write!(f, "({}, {})", self.x, self.y),
        }
    }
}

impl<P: TECurveConfig> Debug for Affine<P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.is_zero() {
            true => write!(f, "infinity"),
            false => write!(f, "({}, {})", self.x, self.y),
        }
    }
}

impl<P: TECurveConfig> PartialEq<Projective<P>> for Affine<P> {
    fn eq(&self, other: &Projective<P>) -> bool {
        self.into_group() == *other
    }
}

impl<P: TECurveConfig> Affine<P> {
    /// Construct a new group element without checking whether the coordinates
    /// specify a point in the subgroup.
    pub const fn new_unchecked(x: P::BaseField, y: P::BaseField) -> Self {
        Self { x, y }
    }

    /// Construct a new group element in a way while enforcing that points are
    /// in the prime-order subgroup.
    pub fn new(x: P::BaseField, y: P::BaseField) -> Self {
        let p = Self::new_unchecked(x, y);
        assert!(p.is_on_curve());
        assert!(p.is_in_correct_subgroup_assuming_on_curve());
        p
    }

    /// Construct the identity of the group
    pub const fn zero() -> Self {
        Self::new_unchecked(P::BaseField::ZERO, P::BaseField::ONE)
    }

    /// Is this point the identity?
    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_one()
    }

    /// Checks that the current point is on the elliptic curve.
    pub fn is_on_curve(&self) -> bool {
        let x2 = self.x.square();
        let y2 = self.y.square();

        let lhs = y2 + P::mul_by_a(x2);
        let rhs = P::BaseField::one() + &(P::COEFF_D * &(x2 * &y2));

        lhs == rhs
    }
}

impl<P: TECurveConfig> Affine<P> {
    /// Checks if `self` is in the subgroup having order equaling that of
    /// `P::ScalarField` given it is on the curve.
    pub fn is_in_correct_subgroup_assuming_on_curve(&self) -> bool {
        P::is_in_correct_subgroup_assuming_on_curve(self)
    }
}

impl<P: TECurveConfig> AffineRepr for Affine<P> {
    type BaseField = P::BaseField;
    type Config = P;
    type Group = Projective<P>;
    type ScalarField = P::ScalarField;

    fn xy(&self) -> Option<(Self::BaseField, Self::BaseField)> {
        (!self.is_zero()).then_some((self.x, self.y))
    }

    fn generator() -> Self {
        P::GENERATOR
    }

    fn zero() -> Self {
        Self::new_unchecked(P::BaseField::ZERO, P::BaseField::ONE)
    }

    fn mul_bigint(&self, by: impl BitIteratorBE) -> Self::Group {
        P::mul_affine(self, by)
    }

    /// Multiplies this element by the cofactor and output the
    /// resulting projective element.
    #[must_use]
    fn mul_by_cofactor_to_group(&self) -> Self::Group {
        P::mul_affine(self, Self::Config::COFACTOR)
    }

    /// Performs cofactor clearing.
    /// The default method is simply to multiply by the cofactor.
    /// Some curves can implement a more efficient algorithm.
    fn clear_cofactor(&self) -> Self {
        P::clear_cofactor(self)
    }
}

impl<P: TECurveConfig> Zeroize for Affine<P> {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        self.x.zeroize();
        self.y.zeroize();
    }
}

impl<P: TECurveConfig> Neg for Affine<P> {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new_unchecked(-self.x, self.y)
    }
}

impl<P: TECurveConfig, T: Borrow<Self>> Add<T> for Affine<P> {
    type Output = Projective<P>;

    fn add(self, other: T) -> Self::Output {
        let mut copy = self.into_group();
        copy += other.borrow();
        copy
    }
}

impl<P: TECurveConfig> Add<Projective<P>> for Affine<P> {
    type Output = Projective<P>;

    fn add(self, other: Projective<P>) -> Projective<P> {
        other + self
    }
}

impl<'a, P: TECurveConfig> Add<&'a Projective<P>> for Affine<P> {
    type Output = Projective<P>;

    fn add(self, other: &'a Projective<P>) -> Projective<P> {
        *other + self
    }
}

impl<P: TECurveConfig, T: Borrow<Self>> Sub<T> for Affine<P> {
    type Output = Projective<P>;

    fn sub(self, other: T) -> Self::Output {
        let mut copy = self.into_group();
        copy -= other.borrow();
        copy
    }
}

impl<P: TECurveConfig> Sub<Projective<P>> for Affine<P> {
    type Output = Projective<P>;

    fn sub(self, other: Projective<P>) -> Projective<P> {
        self + (-other)
    }
}

impl<'a, P: TECurveConfig> Sub<&'a Projective<P>> for Affine<P> {
    type Output = Projective<P>;

    fn sub(self, other: &'a Projective<P>) -> Projective<P> {
        self + (-*other)
    }
}

impl<P: TECurveConfig> Default for Affine<P> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<P: TECurveConfig, T: Borrow<P::ScalarField>> Mul<T> for Affine<P> {
    type Output = Projective<P>;

    #[inline]
    fn mul(self, other: T) -> Self::Output {
        self.mul_bigint(other.borrow().into_bigint())
    }
}

// The projective point X, Y, T, Z is represented in the affine
// coordinates as X/Z, Y/Z.
impl<P: TECurveConfig> From<Projective<P>> for Affine<P> {
    fn from(p: Projective<P>) -> Affine<P> {
        if p.is_zero() {
            Affine::zero()
        } else if p.z.is_one() {
            // If Z is one, the point is already normalized.
            Affine::new_unchecked(p.x, p.y)
        } else {
            // Z is nonzero, so it must have an inverse in a field.
            let z_inv = p.z.inverse().unwrap();
            let x = p.x * &z_inv;
            let y = p.y * &z_inv;
            Affine::new_unchecked(x, y)
        }
    }
}
