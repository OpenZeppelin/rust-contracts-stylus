//! This module provides a generic interface and constant
//! functions for big integers.

pub mod uint;

use core::{
    borrow::Borrow,
    fmt::{Debug, Display, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
        Shl, ShlAssign, Shr, ShrAssign,
    },
};

use num_bigint::BigUint;
use num_traits::{ConstZero, Zero};
// use rand::{
//     distributions::{Distribution, Standard},
//     Rng,
// };
use zeroize::Zeroize;

use crate::{bits::BitIteratorBE, const_for, const_modulo, unroll6_for};

pub type Limb = u64;
pub type Limbs<const N: usize> = [Limb; N];
pub type WideLimb = u128;

// TODO#q: Refactor types to:
//  - Odd<Uint<N>> - Odd numbers. (odd.rs)
//  - Rename u64 and u128 to Limb and WideLimb
//  - Rename functions *_with_carry to carrying_*

#[inline(always)]
#[doc(hidden)]
pub const fn widening_mul(a: u64, b: u64) -> u128 {
    #[cfg(not(target_family = "wasm"))]
    {
        a as u128 * b as u128
    }
    #[cfg(target_family = "wasm")]
    {
        let a_lo = a as u32 as u64;
        let a_hi = a >> 32;
        let b_lo = b as u32 as u64;
        let b_hi = b >> 32;

        let lolo = (a_lo * b_lo) as u128;
        let lohi = ((a_lo * b_hi) as u128) << 32;
        let hilo = ((a_hi * b_lo) as u128) << 32;
        let hihi = ((a_hi * b_hi) as u128) << 64;
        (lolo | hihi) + (lohi + hilo)
    }
}

// TODO#q: we need carrying_mac and mac

/// Calculate a + b * c, returning the lower 64 bits of the result and setting
/// `carry` to the upper 64 bits.
#[inline(always)]
#[doc(hidden)]
pub const fn mac(a: u64, b: u64, c: u64) -> (u64, u64) {
    let tmp = (a as u128) + widening_mul(b, c);
    let carry = (tmp >> 64) as u64;
    (tmp as u64, carry)
}

/// Calculate a + (b * c) + carry, returning the least significant digit
/// and setting carry to the most significant digit.
#[inline(always)]
#[doc(hidden)]
pub const fn carrying_mac(a: u64, b: u64, c: u64, carry: u64) -> (u64, u64) {
    let tmp = (a as u128) + widening_mul(b, c) + (carry as u128);
    let carry = (tmp >> 64) as u64;
    (tmp as u64, carry)
}

pub const fn ct_mac_with_carry(
    a: Limb,
    b: Limb,
    c: Limb,
    carry: Limb,
) -> (Limb, Limb) {
    let a = a as WideLimb;
    let b = b as WideLimb;
    let c = c as WideLimb;
    let carry = carry as WideLimb;
    let ret = a + (b * c) + carry;
    (ret as Limb, (ret >> Limb::BITS) as Limb)
}

/// Calculate a + b * c, discarding the lower 64 bits of the result and setting
/// `carry` to the upper 64 bits.
#[inline(always)]
#[doc(hidden)]
pub fn mac_discard(a: u64, b: u64, c: u64, carry: &mut u64) {
    let tmp = (a as u128) + widening_mul(b, c);
    *carry = (tmp >> 64) as u64;
}

// TODO#q: adc can be unified with adc_for_add_with_carry
/// Calculate `a = a + b + carry` and return the result and carry.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let tmp = a as u128 + b as u128 + carry as u128;
    let carry = (tmp >> 64) as u64;
    (tmp as u64, carry)
}

/// Sets a = a + b + carry, and returns the new carry.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub fn adc_for_add_with_carry(a: &mut u64, b: u64, carry: bool) -> bool {
    let (sum, carry1) = a.overflowing_add(b);
    let (sum, carry2) = sum.overflowing_add(carry as u64);
    *a = sum;
    carry1 | carry2
}

// TODO#q: sbb can be unified with sbb_for_sub_with_borrow
/// Calculate `a = a - b - borrow` and return the result and borrow.
pub const fn sbb(a: u64, b: u64, borrow: u64) -> (u64, u64) {
    let tmp = (1u128 << 64) + (a as u128) - (b as u128) - (borrow as u128);
    let borrow = if tmp >> 64 == 0 { 1 } else { 0 };
    (tmp as u64, borrow)
}

/// Sets a = a - b - borrow, and returns the borrow.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub fn sbb_for_sub_with_borrow(a: &mut u64, b: u64, borrow: bool) -> bool {
    let (sub, borrow1) = a.overflowing_sub(b);
    let (sub, borrow2) = sub.overflowing_sub(borrow as u64);
    *a = sub;
    borrow1 | borrow2
}

// ----------- Traits Impls -----------

// TODO#q: Implement rand Distribution
/*impl<const N: usize> Distribution<BigInt<N>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BigInt<N> {
        BigInt([(); N].map(|_| rng.gen()))
    }
}*/

// TODO#q: implement conversions in as similar way to
// impl_try_from_upper_bounded!(u128 => u8, u16, u32, u64);  as in std
/*
impl<const N: usize> From<u128> for BigInt<N> {
    fn from(value: u128) -> Self {
        let result = Limb::try_from(value);
        if u128::BITS > BigInt::BITS {
            panic!("u128 is too large to fit in BigInt");
        }
    }
}

impl<const N: usize> TryFrom<u128> for BigInt<N> {
    type Error = TryFromIntError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        if u128::BITS > BigInt::BITS {
            Limb::try_from(value).map(|limb| limb.into())
        } else {
            unimplemented!()
        }
    }
}
*/

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
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
    + BitXorAssign<Self>
    + for<'a> BitXorAssign<&'a Self>
    + BitXor<Self, Output=Self>
    + for<'a> BitXor<&'a Self, Output=Self>
    + BitAndAssign<Self>
    + for<'a> BitAndAssign<&'a Self>
    + BitAnd<Self, Output=Self>
    + for<'a> BitAnd<&'a Self, Output=Self>
    + BitOrAssign<Self>
    + for<'a> BitOrAssign<&'a Self>
    + BitOr<Self, Output=Self>
    + for<'a> BitOr<&'a Self, Output=Self>
    + Shr<u32, Output=Self> // TODO#q: use usize instead of u32
    + ShrAssign<u32>
    + Shl<u32, Output=Self>
    + ShlAssign<u32>
{
    /// Number of `usize` limbs representing `Self`.
    const NUM_LIMBS: usize;

    /// Number of bytes in the integer.
    const BYTES: usize = Self::NUM_LIMBS * Limb::BITS as usize / 8;

    /// Returns true if this number is odd.
    /// # Example
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
    /// # Example
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
    /// # Example
    ///
    /// ```
    /// use openzeppelin_crypto::arithmetic::{BigInteger, uint::U64};
    ///
    /// let mut zero = U64::from(0u64);
    /// assert!(zero.is_zero());
    /// ```
    fn is_zero(&self) -> bool;

    /// Compute the minimum number of bits needed to encode this number.
    /// # Example
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

    /// Compute the `i`-th bit of `self`.
    /// # Example
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
    /// Panic if the number of bytes is not equal to `Self::BYTES`.
    fn from_bytes_le(bytes: &[u8]) -> Self;

    /// Convert bigint to little-endian bytes.
    fn into_bytes_le(self) -> alloc::vec::Vec<u8>;
}

// TODO#q: move mul / add operations to BigInt impl

/// Computes `lhs * rhs`, returning the low and the high limbs of the result.
#[inline(always)]
pub const fn ct_mul_wide(lhs: Limb, rhs: Limb) -> (Limb, Limb) {
    let a = lhs as WideLimb;
    let b = rhs as WideLimb;
    let ret = a * b;
    (ret as Limb, (ret >> Limb::BITS) as Limb)
}

// TODO#q: merge with adc function
/// Computes `lhs + rhs + carry`, returning the result along with the new carry
/// (0, 1, or 2).
// NOTE#q: crypto_bigint
#[inline(always)]
pub const fn ct_adc(lhs: Limb, rhs: Limb, carry: Limb) -> (Limb, Limb) {
    // We could use `Word::overflowing_add()` here analogous to
    // `overflowing_add()`, but this version seems to produce a slightly
    // better assembly.
    let a = lhs as WideLimb;
    let b = rhs as WideLimb;
    let carry = carry as WideLimb;
    let ret = a + b + carry;
    (ret as Limb, (ret >> Limb::BITS) as Limb)
}
