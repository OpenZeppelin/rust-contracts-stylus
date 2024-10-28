use core::{
    fmt::{Debug, Display, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl,
        ShlAssign, Shr, ShrAssign,
    },
};

use crypto_bigint::{Integer, Uint, Zero};
use zeroize::Zeroize;

/// Defines a big integer with finite length.
pub trait BigInteger:
    'static
    + Copy
    + Clone
    + Debug
    + Default
    + Display
    + Eq
    + Ord
    + Send
    + Sized
    + Sync
    + Zeroize
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
    + BitXorAssign<Self>
    + for<'a> BitXorAssign<&'a Self>
    + BitXor<Self, Output = Self>
    + for<'a> BitXor<&'a Self, Output = Self>
    + BitAndAssign<Self>
    + for<'a> BitAndAssign<&'a Self>
    + BitAnd<Self, Output = Self>
    + for<'a> BitAnd<&'a Self, Output = Self>
    + BitOrAssign<Self>
    + for<'a> BitOrAssign<&'a Self>
    + BitOr<Self, Output = Self>
    + for<'a> BitOr<&'a Self, Output = Self>
    + Shr<usize, Output = Self>
    + ShrAssign<usize>
    + Shl<usize, Output = Self>
    + ShlAssign<usize>
{
    /// Number of `usize` limbs representing `Self`.
    const NUM_LIMBS: usize;

    /// Multiplies this [`BigInteger`] by another `BigInteger`, storing the
    /// result in `self`. Overflow is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let mut a = B::from(42u64);
    /// let b = B::from(3u64);
    /// assert_eq!(a.mul_low(&b), B::from(126u64));
    ///
    /// // Edge-Case
    /// let mut zero = B::from(0u64);
    /// assert_eq!(zero.mul_low(&B::from(5u64)), B::from(0u64));
    /// ```
    fn mul_low(&self, other: &Self) -> Self;

    /// Multiplies this [`BigInteger`] by another `BigInteger`, returning the
    /// high bits of the result.
    ///
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let (one, x) = (B::from(1u64), B::from(2u64));
    /// let r = x.mul_high(&one);
    /// assert_eq!(r, B::from(0u64));
    ///
    /// // Edge-Case
    /// let mut x = B::from(u64::MAX);
    /// let r = x.mul_high(&B::from(2u64));
    /// assert_eq!(r, B::from(1u64))
    /// ```
    fn mul_high(&self, other: &Self) -> Self;

    /// Multiplies this [`BigInteger`] by another `BigInteger`, returning both
    /// low and high bits of the result.
    ///
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let mut a = B::from(42u64);
    /// let b = B::from(3u64);
    /// let (low_bits, high_bits) = a.mul(&b);
    /// assert_eq!(low_bits, B::from(126u64));
    /// assert_eq!(high_bits, B::from(0u64));
    ///
    /// // Edge-Case
    /// let mut x = B::from(u64::MAX);
    /// let mut max_plus_max = x;
    /// max_plus_max.add_with_carry(&x);
    /// let (low_bits, high_bits) = x.mul(&B::from(2u64));
    /// assert_eq!(low_bits, max_plus_max);
    /// assert_eq!(high_bits, B::from(1u64));
    /// ```
    fn mul(&self, other: &Self) -> (Self, Self);

    /// Returns true iff this number is odd.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let mut one = B::from(1u64);
    /// assert!(one.is_odd());
    /// ```
    fn is_odd(&self) -> bool;

    /// Returns true iff this number is even.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let mut two = B::from(2u64);
    /// assert!(two.is_even());
    /// ```
    fn is_even(&self) -> bool;

    /// Returns true iff this number is zero.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let mut zero = B::from(0u64);
    /// assert!(zero.is_zero());
    /// ```
    fn is_zero(&self) -> bool;

    /// Compute the minimum number of bits needed to encode this number.
    /// # Example
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let zero = B::from(0u64);
    /// assert_eq!(zero.num_bits(), 0);
    /// let one = B::from(1u64);
    /// assert_eq!(one.num_bits(), 1);
    /// let max = B::from(u64::MAX);
    /// assert_eq!(max.num_bits(), 64);
    /// let u32_max = B::from(u32::MAX as u64);
    /// assert_eq!(u32_max.num_bits(), 32);
    /// ```
    fn num_bits(&self) -> usize;

    /// Compute the `i`-th bit of `self`.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let mut one = B::from(1u64);
    /// assert!(one.get_bit(0));
    /// assert!(!one.get_bit(1));
    /// ```
    fn get_bit(&self, i: usize) -> bool;
}

impl<const N: usize> BigInteger for Uint<N> {
    const NUM_LIMBS: usize = N;

    fn mul_low(&self, other: &Self) -> Self {
        self.mul_wide(other).0
    }

    fn mul_high(&self, other: &Self) -> Self {
        self.mul_wide(other).1
    }

    fn mul(&self, other: &Self) -> (Self, Self) {
        self.mul_wide(other)
    }

    fn is_odd(&self) -> bool {
        <Uint<N> as Integer>::is_odd(self).into()
    }

    fn is_even(&self) -> bool {
        <Uint<N> as Integer>::is_even(self).into()
    }

    fn is_zero(&self) -> bool {
        <Uint<N> as Zero>::is_zero(self).into()
    }

    fn num_bits(&self) -> usize {
        self.bits()
    }

    fn get_bit(&self, i: usize) -> bool {
        self.bit(i).into()
    }
}
