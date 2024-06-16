//! Field arithmetic for Nist's P-256 curve.

use core::{
    fmt,
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use bigint::{Integer, U256};

use super::{curve::Curve, p256::P256};

/// This trait represents an element of a field.
pub trait Field:
    Sized
    + Eq
    + Copy
    + Clone
    + Default
    + Send
    + Sync
    + fmt::Debug
    + 'static
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Sum<&'a Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
{
    /// The zero element of the field, the additive identity.
    const ZERO: Self;

    /// The one element of the field, the multiplicative identity.
    const ONE: Self;

    /// Returns true iff this element is zero.
    fn is_zero(&self) -> bool;

    /// Doubles this element.
    #[must_use]
    fn double(&self) -> Self;

    /// Squares this element.
    #[must_use]
    fn square(&self) -> Self;

    /// Cubes this element.
    #[must_use]
    fn cube(&self) -> Self {
        self.square() * self
    }

    /// Computes the multiplicative inverse of this element,
    /// failing if the element is zero.
    #[must_use]
    fn invert(&self) -> Option<Self>;
}

/// An element in the subgroup with base point [`P256::GENERATOR`].
#[derive(Clone, Copy, Debug)]
pub struct FieldElement(pub U256);

impl FieldElement {
    /// Multiplicative identity.
    pub const ONE: Self = FieldElement(U256::ONE);
    /// Zero element.
    pub const ZERO: Self = FieldElement(U256::ZERO);

    /// Convert a `u64` into a [`FieldElement`].
    #[must_use]
    pub const fn from_u64(w: u64) -> Self {
        Self(U256::from_u64(w))
    }

    /// Parse a [`FieldElement`] from big endian hex-encoded bytes.
    ///
    /// Does *not* perform a check that the field element does not overflow the
    /// order.
    ///
    /// This method is primarily intended for defining internal constants.
    pub(crate) const fn from_hex(hex: &str) -> Self {
        Self(U256::from_be_hex(hex))
    }

    /// Determine if this `FieldElement` is odd in the SEC1 sense:
    /// `self mod 2 == 1`.
    #[must_use]
    pub fn is_odd(&self) -> bool {
        self.0.is_odd().into()
    }

    /// Returns `self + rhs mod p`.
    #[must_use]
    pub const fn add(&self, rhs: &Self) -> Self {
        Self(self.0.add_mod(&rhs.0, &P256::ORDER))
    }

    /// Returns `self - rhs mod p`.
    #[must_use]
    pub const fn sub(&self, rhs: &Self) -> Self {
        Self(self.0.sub_mod(&rhs.0, &P256::ORDER))
    }

    /// Negate element.
    #[must_use]
    pub const fn neg(&self) -> Self {
        Self::sub(&Self::ZERO, self)
    }

    /// Returns `self * self mod p`
    #[must_use]
    pub fn square(&self) -> Self {
        self.mul(self)
    }

    /// Returns `self^(2^n) mod p`.
    fn sqn(&self, n: usize) -> Self {
        let mut x = *self;
        let mut i = 0;
        while i < n {
            x = x.square();
            i += 1;
        }
        x
    }

    /// Returns the multiplicative inverse of `self`, if `self` is non-zero.
    #[must_use]
    pub fn invert(&self) -> Option<Self> {
        (!self.is_zero()).then(|| self.invert_unchecked())
    }

    /// Returns the multiplicative inverse of `self`.
    ///
    /// Does not check that `self` is non-zero.
    fn invert_unchecked(&self) -> Self {
        let t111 = self.mul(&self.mul(&self.square()).square());
        let t111111 = t111.mul(&t111.sqn(3));
        let x15 = t111111.sqn(6).mul(&t111111).sqn(3).mul(&t111);
        let x16 = x15.square().mul(self);
        let i53 = x16.sqn(16).mul(&x16).sqn(15);
        let x47 = x15.mul(&i53);
        x47.mul(&i53.sqn(17).mul(self).sqn(143).mul(&x47).sqn(47))
            .sqn(2)
            .mul(self)
    }
}

impl Field for FieldElement {
    const ONE: Self = Self::ONE;
    const ZERO: Self = Self::ZERO;

    fn is_zero(&self) -> bool {
        self.0 == FieldElement::ZERO.0
    }

    fn double(&self) -> Self {
        self.add(self)
    }

    fn square(&self) -> Self {
        self.square()
    }

    fn invert(&self) -> Option<Self> {
        self.invert()
    }
}

impl Default for FieldElement {
    fn default() -> Self {
        FieldElement::ZERO
    }
}

impl From<u64> for FieldElement {
    fn from(n: u64) -> FieldElement {
        Self(U256::from(n))
    }
}

impl PartialEq for FieldElement {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FieldElement {}

impl Add<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn add(self, other: FieldElement) -> FieldElement {
        FieldElement::add(&self, &other)
    }
}

impl Add<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn add(self, other: &FieldElement) -> FieldElement {
        FieldElement::add(&self, other)
    }
}

impl Add<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn add(self, other: &FieldElement) -> FieldElement {
        FieldElement::add(self, other)
    }
}

impl AddAssign<FieldElement> for FieldElement {
    fn add_assign(&mut self, other: FieldElement) {
        *self = FieldElement::add(self, &other);
    }
}

impl AddAssign<&FieldElement> for FieldElement {
    fn add_assign(&mut self, other: &FieldElement) {
        *self = FieldElement::add(self, other);
    }
}

impl Sub<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn sub(self, other: FieldElement) -> FieldElement {
        FieldElement::sub(&self, &other)
    }
}

impl Sub<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn sub(self, other: &FieldElement) -> FieldElement {
        FieldElement::sub(&self, other)
    }
}

impl Sub<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn sub(self, other: &FieldElement) -> FieldElement {
        FieldElement::sub(self, other)
    }
}

impl SubAssign<FieldElement> for FieldElement {
    fn sub_assign(&mut self, other: FieldElement) {
        *self = FieldElement::sub(self, &other);
    }
}

impl SubAssign<&FieldElement> for FieldElement {
    fn sub_assign(&mut self, other: &FieldElement) {
        *self = FieldElement::sub(self, other);
    }
}

impl Mul<FieldElement> for FieldElement {
    type Output = FieldElement;

    fn mul(self, other: FieldElement) -> FieldElement {
        FieldElement(self.0.mul_mod(&other.0, &P256::ORDER))
    }
}

impl Mul<&FieldElement> for FieldElement {
    type Output = FieldElement;

    fn mul(self, other: &FieldElement) -> FieldElement {
        FieldElement(self.0.mul_mod(&other.0, &P256::ORDER))
    }
}

impl Mul<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, other: &FieldElement) -> FieldElement {
        FieldElement(self.0.mul_mod(&other.0, &P256::ORDER))
    }
}

impl MulAssign<FieldElement> for FieldElement {
    fn mul_assign(&mut self, other: FieldElement) {
        *self = FieldElement(self.0.mul_mod(&other.0, &P256::ORDER));
    }
}

impl MulAssign<&FieldElement> for FieldElement {
    fn mul_assign(&mut self, other: &FieldElement) {
        *self = FieldElement(self.0.mul_mod(&other.0, &P256::ORDER));
    }
}

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        FieldElement::ZERO - self
    }
}

impl Neg for &FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        FieldElement::ZERO - self
    }
}

impl Sum for FieldElement {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(Add::add).unwrap_or(Self::ZERO)
    }
}

impl<'a> Sum<&'a FieldElement> for FieldElement {
    fn sum<I: Iterator<Item = &'a FieldElement>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Mul;

    #[cfg(target_pointer_width = "64")]
    use bigint::U256;
    #[cfg(target_pointer_width = "64")]
    use proptest::{num::u64::ANY, prelude::*};

    use super::{Field, FieldElement};
    use crate::elliptic_curve::test_vectors::field::DBL_TEST_VECTORS;

    #[test]
    fn zero_is_additive_identity() {
        let zero = FieldElement::ZERO;
        let one = FieldElement::ONE;
        assert_eq!(zero.add(&zero), zero);
        assert_eq!(one.add(&zero), one);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let one = FieldElement::ONE;
        assert_eq!(one.mul(&one), one);
    }

    #[test]
    fn repeated_add() {
        let mut r = FieldElement::ONE;
        for i in 0..DBL_TEST_VECTORS.len() {
            let item = FieldElement::from_hex(DBL_TEST_VECTORS[i]);
            assert_eq!(r, item);
            r = r + &r;
        }
    }

    #[test]
    fn repeated_double() {
        let mut r = FieldElement::ONE;
        for i in 0..DBL_TEST_VECTORS.len() {
            let item = FieldElement::from_hex(DBL_TEST_VECTORS[i]);
            assert_eq!(r, item);
            r = r.double();
        }
    }

    #[test]
    fn multiply() {
        let one = FieldElement::ONE;
        let two = one + &one;
        let three = two + &one;
        let six = three + &three;
        assert_eq!(six, two * &three);

        let minus_two = -two;
        let minus_three = -three;
        assert_eq!(two, -minus_two);

        assert_eq!(minus_three * &minus_two, minus_two * &minus_three);
        assert_eq!(six, minus_two * &minus_three);
    }

    #[test]
    fn repeated_mul() {
        let mut r = FieldElement::ONE;
        let two = r + &r;
        for i in 0..DBL_TEST_VECTORS.len() {
            let item = FieldElement::from_hex(DBL_TEST_VECTORS[i]);
            assert_eq!(r, item);
            r = r * &two;
        }
    }

    #[test]
    fn negation() {
        let two = FieldElement::ONE.double();
        let neg_two = -two;
        assert_eq!(two + &neg_two, FieldElement::ZERO);
        assert_eq!(-neg_two, two);
    }

    #[test]
    fn invert() {
        assert!(bool::from(FieldElement::ZERO.invert().is_none()));

        let one = FieldElement::ONE;
        assert_eq!(one.invert().unwrap(), one);

        let two = one + &one;
        let inv_two = two.invert().unwrap();
        assert_eq!(two * &inv_two, one);

        let three = one + &one + &one;
        let inv_three = three.invert().unwrap();
        assert_eq!(three * &inv_three, one);

        let minus_three = -three;
        let inv_minus_three = minus_three.invert().unwrap();
        assert_eq!(inv_minus_three, -inv_three);
        assert_eq!(three * &inv_minus_three, -one);
    }

    #[cfg(target_pointer_width = "64")]
    proptest! {
        /// This checks behaviour well within the field ranges, because it
        /// doesn't set the highest limb.
        #[test]
        fn add_then_sub(
            a0 in ANY,
            a1 in ANY,
            a2 in ANY,
            b0 in ANY,
            b1 in ANY,
            b2 in ANY,
        ) {
            let a = FieldElement(U256::from_words([a0, a1, a2, 0]));
            let b = FieldElement(U256::from_words([b0, b1, b2, 0]));
            assert_eq!(a.add(&b).sub(&a), b);
        }
    }
}
