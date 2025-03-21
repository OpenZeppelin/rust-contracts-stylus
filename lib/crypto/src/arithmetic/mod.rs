//! This module provides a generic interface, constant functions and common
//! arithmetics for big integers.

pub mod limb;
pub mod uint;

use core::{
    fmt::{Debug, Display},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

use limb::Limb;
use zeroize::Zeroize;

use crate::bits::BitIteratorBE;

/// Defines a big integer with a constant length.
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
    + From<u128>
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
    + BitIteratorBE
{
    /// Number of `usize` limbs representing `Self`.
    const NUM_LIMBS: usize;

    /// Number of bits in the integer.
    const BITS: usize = Self::LIMB_BITS * Self::NUM_LIMBS;

    /// Number of bits in a limb.
    const LIMB_BITS: usize;

    /// The largest value that can be represented by this integer type.
    const MAX: Self;

    /// The multiplicative identity element of Self, 1.
    const ONE: Self;

    /// The additive identity element of Self, 0.
    const ZERO: Self;

    /// Returns true if this number is odd.
    ///
    /// # Examples
    ///
    /// ```
    /// use openzeppelin_crypto::arithmetic::{BigInteger, uint::U64};
    ///
    /// let mut one = U64::from(1u64);
    /// assert!(one.is_odd());
    /// ```
    fn is_odd(&self) -> bool;

    /// Returns true if this number is even.
    ///
    /// # Examples
    ///
    /// ```
    /// use openzeppelin_crypto::arithmetic::{BigInteger, uint::U64};
    ///
    /// let mut two = U64::from(2u64);
    /// assert!(two.is_even());
    /// ```
    fn is_even(&self) -> bool;

    /// Returns true if this number is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use openzeppelin_crypto::arithmetic::{BigInteger, uint::U64};
    ///
    /// let mut zero = U64::from(0u64);
    /// assert!(zero.is_zero());
    /// ```
    fn is_zero(&self) -> bool;

    /// Return the minimum number of bits needed to encode this number.
    ///
    /// # Examples
    /// ```
    /// use openzeppelin_crypto::arithmetic::{BigInteger, uint::U64};
    ///
    /// let zero = U64::from(0u64);
    /// assert_eq!(zero.num_bits(), 0);
    /// let one = U64::from(1u64);
    /// assert_eq!(one.num_bits(), 1);
    /// let max = U64::from(u64::MAX);
    /// assert_eq!(max.num_bits(), 64);
    /// let u32_max = U64::from(u32::MAX as u64);
    /// assert_eq!(u32_max.num_bits(), 32);
    /// ```
    fn num_bits(&self) -> usize;

    /// Find the `i`-th bit of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use openzeppelin_crypto::arithmetic::{BigInteger, uint::U64};
    ///
    /// let mut one = U64::from(1u64);
    /// assert!(one.get_bit(0));
    /// assert!(!one.get_bit(1));
    /// ```
    fn get_bit(&self, i: usize) -> bool;

    /// Create bigint from little-endian bytes.
    ///
    /// # Panics
    ///
    /// * If the number of bytes is not equal to `Self::BYTES`.
    fn from_bytes_le(bytes: &[u8]) -> Self;

    /// Convert bigint to little-endian bytes.
    fn into_bytes_le(self) -> alloc::vec::Vec<u8>;
}
