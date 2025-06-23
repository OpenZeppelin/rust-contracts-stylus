//! This module provides common arithmetics to work with finite fields.
//! Implementations of some used fields provided in the [`instance`]
//! module.
//!
//! Abstractions and api in this module are similar to Arkworks Algebra [ark-ff
//! library].
//!
//! Here is an example operations over a prime finite field (aka Fp) with a
//! prime modulus `17` and generator element `3`.
//!
//! # Examples
//!
//! ```rust
//! use openzeppelin_crypto::{
//!     arithmetic::uint::U64,
//!     field::{
//!         fp::{Fp64, FpParams, LIMBS_64},
//!         group::AdditiveGroup,
//!         Field,
//!     },
//!     fp_from_num,
//!     from_num,
//! };
//!
//! pub type ExampleField = Fp64<FpParam>;
//! pub struct FpParam;
//! impl FpParams<LIMBS_64> for FpParam {
//!     const MODULUS: U64 = from_num!("17");
//!     const GENERATOR: Fp64<FpParam> = fp_from_num!("3");
//! }
//!
//! # fn main() {
//! let a = ExampleField::from(9);
//! let b = ExampleField::from(10);
//!
//! assert_eq!(a, ExampleField::from(26));          // 26 =  9 mod 17
//! assert_eq!(a - b, ExampleField::from(16));      // -1 = 16 mod 17
//! assert_eq!(a + b, ExampleField::from(2));       // 19 =  2 mod 17
//! assert_eq!(a * b, ExampleField::from(5));       // 90 =  5 mod 17
//! assert_eq!(a.square(), ExampleField::from(13)); // 81 = 13 mod 17
//! assert_eq!(b.double(), ExampleField::from(3));  // 20 =  3 mod 17
//! assert_eq!(a / b, a * b.inverse().unwrap());    // need to unwrap since `b` could be 0 which is not invertible
//! # }
//! ```
//!
//! [ark-ff library]: https://github.com/arkworks-rs/algebra/tree/master/ff
use core::{
    fmt::{Debug, Display},
    hash::Hash,
    iter::Product,
    ops::{Div, DivAssign, Neg},
};

use group::AdditiveGroup;
use num_traits::{One, Zero};
use zeroize::Zeroize;

use crate::bits::BitIteratorBE;

pub mod fp;
pub mod group;
pub mod instance;
pub mod prime;

/// Defines an abstract field.
/// Types implementing [`Field`] support common field operations such as
/// addition, subtraction, multiplication, and inverses.
pub trait Field:
    'static
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + Send
    + Sync
    + Eq
    + Zero
    + One
    + Ord
    + Neg<Output = Self>
    + Zeroize
    + Sized
    + Hash
    + AdditiveGroup<Scalar = Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + for<'a> Div<&'a Self, Output = Self>
    + for<'a> DivAssign<&'a Self>
    + for<'a> Div<&'a mut Self, Output = Self>
    + for<'a> DivAssign<&'a mut Self>
    + for<'a> Product<&'a Self>
    + From<u128>
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
    + From<i128>
    + From<i64>
    + From<i32>
    + From<i16>
    + From<i8>
    + From<bool>
    + Product<Self>
{
    /// The multiplicative identity of the field.
    const ONE: Self;

    /// Returns the extension degree of this field.
    #[must_use]
    fn extension_degree() -> usize;

    /// Returns `self * self`.
    #[must_use]
    fn square(&self) -> Self;

    /// Squares `self` in place.
    fn square_in_place(&mut self) -> &mut Self;

    /// Computes the multiplicative inverse of `self` if `self` is nonzero.
    fn inverse(&self) -> Option<Self>;

    /// If `self.inverse().is_none()`, this just returns `None`. Otherwise, it
    /// sets `self` to `self.inverse().unwrap()`.
    fn inverse_in_place(&mut self) -> Option<&mut Self>;

    /// Returns `self^exp`, where `exp` is an integer.
    ///
    /// NOTE: Consumers should pass `exp`'s type `S` with the least bit size
    /// possible.
    /// e.g. for `pow(12)` u8 type is small enough to represent `12`.
    #[must_use]
    fn pow<S: BitIteratorBE>(&self, exp: S) -> Self {
        // Variant `Option::<Self>::None` corresponds to `one`.
        // This approach removes pointless multiplications by one, that
        // are still expensive.
        let mut res: Option<Self> = None;

        for has_bit in exp.bit_be_trimmed_iter() {
            // If res is not empty, square it.
            if let Some(res) = &mut res {
                res.square_in_place();
            }

            // If bit is set,
            if has_bit {
                match res {
                    None => {
                        // and res is empty, set it to self.
                        res = Some(*self);
                    }
                    Some(ref mut res) => {
                        // and res is not empty, multiply it by self.
                        *res *= self;
                    }
                }
            }
        }

        // If res is empty, return one.
        res.unwrap_or(Self::ONE)
    }

    /// Returns `sum([a_i * b_i])`.
    #[inline]
    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        let mut sum = Self::zero();
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }
}
