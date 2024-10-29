//! TODO#q: docs: describe module
//! ## Example
//! TODO#q: docs: dd same kind of example but for Fp
//! ```rust
//! use ark_ff::{AdditiveGroup, fields::{Field, Fp64, MontBackend, MontConfig}};
//!
//! #[derive(MontConfig)]
//! #[modulus = "17"]
//! #[generator = "3"]
//! pub struct FqConfig;
//! pub type Fq = Fp64<MontBackend<FqConfig, 1>>;
//!
//! # fn main() {
//! let a = Fq::from(9);
//! let b = Fq::from(10);
//!
//! assert_eq!(a, Fq::from(26));          // 26 =  9 mod 17
//! assert_eq!(a - b, Fq::from(16));      // -1 = 16 mod 17
//! assert_eq!(a + b, Fq::from(2));       // 19 =  2 mod 17
//! assert_eq!(a * b, Fq::from(5));       // 90 =  5 mod 17
//! assert_eq!(a.square(), Fq::from(13)); // 81 = 13 mod 17
//! assert_eq!(b.double(), Fq::from(3));  // 20 =  3 mod 17
//! assert_eq!(a / b, a * b.inverse().unwrap()); // need to unwrap since `b` could be 0 which is not invertible
//! # }
//! ```
use core::{
    fmt::{Debug, Display},
    hash::Hash,
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
    },
};

use group::AdditiveGroup;
use num_traits::{One, Zero};
use zeroize::Zeroize;

use crate::bits::BitIteratorBE;

pub mod fp;
pub mod group;
pub mod prime;
pub mod vesta;

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

    // TODO#q: seems we should move it to PrimeField
    /// Returns the characteristic of the field,
    /// in little-endian representation.
    fn characteristic() -> &'static [u64];

    // TODO#q: seems we should move it to PrimeField
    /// Returns the extension degree of this field with respect
    /// to `Self::BasePrimeField`.
    fn extension_degree() -> u64;

    /// Returns `self * self`.
    #[must_use]
    fn square(&self) -> Self;

    /// Squares `self` in place.
    fn square_in_place(&mut self) -> &mut Self;

    /// Computes the multiplicative inverse of `self` if `self` is nonzero.
    #[must_use]
    fn inverse(&self) -> Option<Self>;

    /// If `self.inverse().is_none()`, this just returns `None`. Otherwise, it
    /// sets `self` to `self.inverse().unwrap()`.
    fn inverse_in_place(&mut self) -> Option<&mut Self>;

    /// Returns `self^exp`, where `exp` is an integer represented with `u64`
    /// limbs.
    /// Least significant limb first.
    #[must_use]
    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::one();

        for i in exp.bit_be_trimmed_iter() {
            res.square_in_place();

            if i {
                res *= self;
            }
        }
        res
    }
}
