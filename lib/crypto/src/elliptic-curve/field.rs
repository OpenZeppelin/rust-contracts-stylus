//! Field arithmetic modulo p = 2^{224}(2^{32} − 1) + 2^{192} + 2^{96} − 1

#![allow(clippy::assign_op_pattern, clippy::op_ref)]

use bn::{Integer, U256};

use core::{
    iter::Sum,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

const MODULUS_HEX: &str =
    "ffffffff00000001000000000000000000000000ffffffffffffffffffffffff";

/// Constant representing the modulus
/// p = 2^{224}(2^{32} − 1) + 2^{192} + 2^{96} − 1
pub const MODULUS: FieldElement = FieldElement(U256::from_be_hex(MODULUS_HEX));

/// R = 2^256 mod p
const R: FieldElement = FieldElement(U256::from_be_hex(
    "00000000fffffffeffffffffffffffffffffffff000000000000000000000001",
));

/// R^2 = 2^512 mod p
const R2: FieldElement = FieldElement(U256::from_be_hex(
    "00000004fffffffdfffffffffffffffefffffffbffffffff0000000000000003",
));

/// An element in the finite field modulo p = 2^{224}(2^{32} − 1) + 2^{192} + 2^{96} − 1.
///
/// The internal representation is in little-endian order. Elements are always in
/// Montgomery form; i.e., FieldElement(a) = aR mod p, with R = 2^256.
#[derive(Clone, Copy, Debug)]
pub struct FieldElement(pub U256);

impl FieldElement {
    /// Zero element.
    pub const ZERO: Self = FieldElement(U256::ZERO);

    /// Multiplicative identity.
    pub const ONE: Self = R;

    /// Convert a `u64` into a [`FieldElement`].
    pub const fn from_u64(w: u64) -> Self {
        Self(U256::from_u64(w))
    }

    /// Parse a [`FieldElement`] from big endian hex-encoded bytes.
    ///
    /// Does *not* perform a check that the field element does not overflow the order.
    ///
    /// This method is primarily intended for defining internal constants.
    pub(crate) const fn from_hex(hex: &str) -> Self {
        Self(U256::from_be_hex(hex))
    }

    /// Determine if this `FieldElement` is zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> bool {
        self.0 == FieldElement::ZERO.0
    }

    /// Determine if this `FieldElement` is odd in the SEC1 sense: `self mod 2 == 1`.
    pub fn is_odd(&self) -> bool {
        self.0.is_odd().into()
    }

    /// Returns self + rhs mod p
    pub const fn add(&self, rhs: &Self) -> Self {
        Self(self.0.add_mod(&rhs.0, &MODULUS.0))
    }

    /// Returns 2 * self.
    pub const fn double(&self) -> Self {
        self.add(self)
    }

    /// Returns self - rhs mod p
    pub const fn sub(&self, rhs: &Self) -> Self {
        Self(self.0.sub_mod(&self.0, &rhs.0))
    }

    /// Negate element.
    pub const fn neg(&self) -> Self {
        Self::sub(&Self::ZERO, self)
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

impl Neg for FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        FieldElement::ZERO - &self
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
