use core::fmt::{Debug, Display};
use std::{
    hash::Hash,
    iter::Sum,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use num_traits::Zero;
use zeroize::Zeroize;

use crate::field::Field;

/// Defines an abstract group with additive notation.
/// Support addition and subtraction with itself and multiplication by scalar.
/// Scalar and group can be different types.
///
/// E.g., Points on an elliptic curve define an additive group and can be
/// multiplied by a scalar.
pub trait AdditiveGroup:
    Eq
    + 'static
    + Sized
    + Copy
    + Clone
    + Default
    + Send
    + Sync
    + Hash
    + Debug
    + Display
    + Zeroize
    + Zero
    + Neg<Output = Self>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<<Self as AdditiveGroup>::Scalar, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<<Self as AdditiveGroup>::Scalar>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a <Self as AdditiveGroup>::Scalar, Output = Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a <Self as AdditiveGroup>::Scalar>
    + for<'a> Add<&'a mut Self, Output = Self>
    + for<'a> Sub<&'a mut Self, Output = Self>
    + for<'a> Mul<&'a mut <Self as AdditiveGroup>::Scalar, Output = Self>
    + for<'a> AddAssign<&'a mut Self>
    + for<'a> SubAssign<&'a mut Self>
    + for<'a> MulAssign<&'a mut <Self as AdditiveGroup>::Scalar>
    + Sum<Self>
    + for<'a> Sum<&'a Self>
{
    /// Scalar associated with the group.
    type Scalar: Field;

    /// Additive identity of the group.
    const ZERO: Self;

    /// Doubles `self`.
    #[must_use]
    fn double(&self) -> Self {
        let mut copy = *self;
        copy.double_in_place();
        copy
    }

    /// Doubles `self` in place.
    fn double_in_place(&mut self) -> &mut Self {
        self.add_assign(*self);
        self
    }

    /// Negates `self` in place.
    fn neg_in_place(&mut self) -> &mut Self {
        *self = -(*self);
        self
    }
}
