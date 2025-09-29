//! This module contains the [`Uint`] unsigned big integer used for
//! cryptographic applications, altogether with its exact implementations
//! [`U64`] for 64 bits, [`U128`] for 128 bits, and so on.

use alloc::vec::Vec;
use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Debug, Display, Result, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not,
        Shl, ShlAssign, Shr, ShrAssign,
    },
};

use num_traits::ConstZero;
use zeroize::Zeroize;

use crate::{
    arithmetic::{
        limb,
        limb::{Limb, Limbs},
        BigInteger,
    },
    bits::BitIteratorBE,
    const_for, const_for_unroll6, const_rev_for,
};

/// Stack-allocated big unsigned integer.
///
/// Generic over number `N` of [`Limb`]s.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Zeroize)]
pub struct Uint<const N: usize> {
    pub(crate) limbs: Limbs<N>,
}

impl<const N: usize> Default for Uint<N> {
    fn default() -> Self {
        Self { limbs: [Limb::ZERO; N] }
    }
}

/// Declare [`Uint`] types for different bit sizes.
macro_rules! declare_num {
    ($num:ident, $bits:expr) => {
        #[doc = "Unsigned integer with "]
        #[doc = stringify!($bits)]
        #[doc = "bits size."]
        pub type $num = $crate::arithmetic::uint::Uint<
            { usize::div_ceil($bits, $crate::arithmetic::Limb::BITS as usize) },
        >;
    };
}

declare_num!(U64, 64);
declare_num!(U128, 128);
declare_num!(U192, 192);
declare_num!(U256, 256);
declare_num!(U384, 384);
declare_num!(U448, 448);
declare_num!(U512, 512);
declare_num!(U576, 576);
declare_num!(U640, 640);
declare_num!(U704, 704);
declare_num!(U768, 768);
declare_num!(U832, 832);

impl<const N: usize> Uint<N> {
    /// Create a new [`Uint`] from the provided `limbs` (constant).
    #[must_use]
    pub const fn new(limbs: [Limb; N]) -> Self {
        Self { limbs }
    }

    /// Returns reference to the inner [`Limbs`] array (constant).
    #[must_use]
    pub const fn as_limbs(&self) -> &Limbs<N> {
        &self.limbs
    }

    /// Returns inner [`Limbs`] array (constant).
    #[must_use]
    pub const fn into_limbs(self) -> Limbs<N> {
        self.limbs
    }

    /// Returns true if this number is odd (constant).
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn is_odd(&self) -> bool {
        self.limbs[0] & 1 == 1
    }

    /// Returns true if this number is even (constant).
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn is_even(&self) -> bool {
        self.limbs[0] & 1 == 0
    }

    /// Checks `self` is greater or equal then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn ge(&self, rhs: &Self) -> bool {
        let mut result = true;
        const_for_unroll6!((i in 0..N) {
            let a = self.limbs[i];
            let b = rhs.limbs[i];
            if a > b {
                result = true;
            } else if a < b {
                result = false;
            }
        });
        result
    }

    /// Checks `self` is greater then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn gt(&self, rhs: &Self) -> bool {
        let mut result = false;
        const_for_unroll6!((i in 0..N) {
            let a = self.limbs[i];
            let b = rhs.limbs[i];
            if a > b {
                result = true;
            } else if a < b {
                result = false;
            }
        });
        result
    }

    /// Checks `self` is less or equal then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn le(&self, rhs: &Self) -> bool {
        let mut result = true;
        const_for_unroll6!((i in 0..N) {
            let a = self.limbs[i];
            let b = rhs.limbs[i];
            if a < b {
                result = true;
            } else if a > b {
                result = false;
            }
        });
        result
    }

    /// Checks `self` is less then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn lt(&self, rhs: &Self) -> bool {
        let mut result = false;
        const_for_unroll6!((i in 0..N) {
            let a = self.limbs[i];
            let b = rhs.limbs[i];
            if a < b {
                result = true;
            } else if a > b {
                result = false;
            }
        });
        result
    }

    /// Checks `self` is zero (constant).
    #[must_use]
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.eq(&Self::ZERO)
    }

    /// Checks if `self` is equal to `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn eq(&self, rhs: &Self) -> bool {
        const_for!((i in 0..N) {
            if self.limbs[i] != rhs.limbs[i] {
                return false;
            }
        });
        true
    }

    /// Checks if `self` is not equal to `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn ne(&self, rhs: &Self) -> bool {
        !self.eq(rhs)
    }

    /// Return the minimum number of bits needed to encode this number.
    ///
    /// One bit is necessary to encode zero.
    #[doc(hidden)]
    #[must_use]
    pub const fn num_bits(&self) -> usize {
        // One bit is necessary to encode zero.
        if self.is_zero() {
            return 1;
        }

        // Total number of bits.
        let mut num_bits = Self::BITS;

        // Start with the last (highest) limb.
        const_rev_for!((index in 0..N) {
            // Subtract leading zeroes, from the total number of limbs.
            let leading = self.limbs[index].leading_zeros() as usize;
            num_bits -= leading;

            // If the limb is not empty, stop processing other limbs.
            if leading != 64 {
                break;
            }
        });

        // And return the result.
        num_bits
    }

    /// Find the `i`-th bit of `self`.
    #[must_use]
    pub const fn get_bit(&self, i: usize) -> bool {
        // If `i` is more than total bits, return `false`.
        if i >= Self::BITS {
            return false;
        }

        // Otherwise find `limb` and `bit` indices and get the bit.
        let bits_in_limb = Limb::BITS as usize;
        let limb = i / bits_in_limb;
        let bit = i - bits_in_limb * limb;
        let mask = 1 << bit;
        (self.limbs[limb] & mask) != 0
    }

    /// Multiplies `self` by `2` in-place, returning whether overflow occurred.
    #[inline(always)]
    #[allow(unused)]
    pub fn checked_mul2_assign(&mut self) -> bool {
        let mut last = 0;
        const_for_unroll6!((i in 0..N) {
            let a = &mut self.limbs[i];
            let tmp = *a >> 63;
            *a <<= 1;
            *a |= last;
            last = tmp;
        });
        last != 0
    }

    /// Multiplies `self` by `2`, returning the result and whether overflow
    /// occurred (constant).
    const fn checked_mul2(mut self) -> (Self, bool) {
        let mut last = 0;
        const_for!((i in 0..N) {
            let a = self.limbs[i];
            let tmp = a >> 63;
            self.limbs[i] <<= 1;
            self.limbs[i] |= last;
            last = tmp;
        });
        (self, last != 0)
    }

    /// Divide `self` by `2` in-place.
    pub fn div2_assign(&mut self) {
        let mut t = 0;
        for a in self.limbs.iter_mut().rev() {
            let t2 = *a << 63;
            *a >>= 1;
            *a |= t;
            t = t2;
        }
    }

    /// Subtract `rhs` from `self`, returning the result and whether overflow
    /// occurred (constant).
    #[inline(always)]
    #[must_use]
    pub const fn checked_sub(mut self, rhs: &Self) -> (Self, bool) {
        let mut borrow = false;

        const_for_unroll6!((i in 0..N) {
            (self.limbs[i], borrow) = limb::sbb(self.limbs[i], rhs.limbs[i], borrow);
        });

        (self, borrow)
    }

    /// Subtract `rhs` from `self`, returning the result wrapping around the
    /// lower boundary (constant).
    #[inline(always)]
    #[must_use]
    pub const fn wrapping_sub(&self, rhs: &Self) -> Self {
        self.checked_sub(rhs).0
    }

    /// Add `rhs` to `self`, returning the result and whether overflow occurred
    /// (constant).
    #[inline]
    #[must_use]
    pub const fn checked_add(mut self, rhs: &Self) -> (Self, bool) {
        let mut carry = false;

        const_for!((i in 0..N) {
            (self.limbs[i], carry) = limb::adc(self.limbs[i], rhs.limbs[i], carry);
        });

        (self, carry)
    }

    /// Add `rhs` to `self` in-place, returning whether overflow occurred.
    #[inline(always)]
    pub fn checked_add_assign(&mut self, rhs: &Self) -> bool {
        let mut carry = false;

        const_for_unroll6!((i in 0..N) {
            carry = limb::adc_assign(&mut self.limbs[i], rhs.limbs[i], carry);
        });

        carry
    }

    /// Subtract `rhs` from `self` in-place, returning whether overflow
    /// occurred.
    #[inline(always)]
    pub fn checked_sub_assign(&mut self, rhs: &Self) -> bool {
        let mut borrow = false;

        const_for_unroll6!((i in 0..N) {
            borrow =
                limb::sbb_assign(&mut self.limbs[i], rhs.limbs[i], borrow);
        });

        borrow
    }

    /// Compute "wide" multiplication, with a product twice the size of the
    /// input.
    ///
    /// Returns a tuple containing the `(lo, hi)` components of the product.
    ///
    /// Basic multiplication algorithm described in [wiki].
    /// It is fast enough for runtime use when optimized with loop "unrolls",
    /// like [`const_for_unroll6`].
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Multiplication_algorithm
    #[inline(always)]
    #[must_use]
    pub const fn widening_mul(&self, rhs: &Self) -> (Self, Self) {
        let (mut lo, mut hi) = ([0u64; N], [0u64; N]);
        // For each digit of the first number,
        const_for_unroll6!((i in 0..N) {
            let mut carry = 0;
            // perform multiplication of each digit from the second.
            const_for_unroll6!((j in 0..N) {
                // And if the multiplication result is too big,
                let k = i + j;
                if k >= N {
                    // it should go to the high (hi) part.
                    (hi[k - N], carry) = limb::carrying_mac(
                        hi[k - N],
                        self.limbs[i],
                        rhs.limbs[j],
                        carry
                    );
                } else {
                    (lo[k], carry) = limb::carrying_mac(
                        lo[k],
                        self.limbs[i],
                        rhs.limbs[j],
                        carry
                    );
                }
            });
            // Set the last carry to the next limb.
            hi[i] = carry;
        });

        (Self::new(lo), Self::new(hi))
    }

    /// Multiply two numbers and panic on overflow.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub const fn mul(&self, rhs: &Self) -> Self {
        let (low, high) = self.widening_mul(rhs);
        assert!(high.eq(&Uint::<N>::ZERO), "overflow on multiplication");
        low
    }

    /// Add two numbers and panic on overflow.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub const fn add(&self, rhs: &Self) -> Self {
        let (low, carry) = self.adc(rhs, false);
        assert!(!carry, "overflow on addition");
        low
    }

    /// Add two numbers wrapping around the upper boundary.
    #[must_use]
    pub const fn wrapping_add(&self, rhs: &Self) -> Self {
        let (low, _) = self.adc(rhs, false);
        low
    }

    /// Computes `a + b + carry`, returning the result along with the new carry.
    #[inline(always)]
    #[must_use]
    pub const fn adc(&self, rhs: &Uint<N>, mut carry: bool) -> (Self, bool) {
        let mut limbs = [Limb::ZERO; N];

        const_for!((i in 0..N) {
            (limbs[i], carry) = limb::adc(self.limbs[i], rhs.limbs[i], carry);
        });

        (Self { limbs }, carry)
    }

    /// Create a new [`Uint`] from the provided little endian bytes.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub const fn from_le_slice(bytes: &[u8]) -> Self {
        const LIMB_BYTES: usize = Limb::BITS as usize / 8;
        assert!(
            bytes.len() == LIMB_BYTES * N,
            "bytes are not the expected size"
        );

        let mut res = [Limb::ZERO; N];
        let mut buf = [0u8; LIMB_BYTES];

        const_for!((i in 0..N) {
            const_for!((j in 0..LIMB_BYTES) {
                buf[j] = bytes[i * LIMB_BYTES + j];
            });
            res[i] = Limb::from_le_bytes(buf);
        });

        Self::new(res)
    }

    /// Construct `Self` from the other [`Uint`] of different size (constant).
    ///
    /// # Panics
    ///
    /// * If `value` is bigger than `Self` maximum size.
    #[must_use]
    pub const fn from_uint<const N2: usize>(value: Uint<N2>) -> Self {
        let mut res = Uint::<N>::ZERO;
        const_for!((i in 0..{value.limbs.len()}) {
            if i < res.limbs.len() {
                res.limbs[i] = value.limbs[i];
            } else if value.limbs[i] != Limb::ZERO {
                panic!("converted element is too large")
            }
        });
        res
    }
}

// ----------- From Impls -----------

/// Constant conversions from primitive types.
macro_rules! impl_from_primitive {
    ($int:ty, $func_name:ident) => {
        impl<const N: usize> Uint<N> {
            #[doc = "Create a [`Uint`] from"]
            #[doc = stringify!($int)]
            #[doc = "integer (constant)."]
            #[must_use]
            #[allow(clippy::cast_lossless)]
            pub const fn $func_name(val: $int) -> Self {
                assert!(N >= 1, "number of limbs must be greater than zero");
                let mut repr = Self::ZERO;
                repr.limbs[0] = val as Limb;
                repr
            }
        }
    };
}
impl_from_primitive!(u8, from_u8);
impl_from_primitive!(u16, from_u16);
impl_from_primitive!(u32, from_u32);
impl_from_primitive!(u64, from_u64);
impl_from_primitive!(usize, from_usize);

// Logic for `u128` conversion is different from `u8`..`u64`, due to the size of
// the `Limb`.
impl<const N: usize> Uint<N> {
    /// Create a [`Uint`] from a `u128` integer (constant).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_lossless)]
    #[allow(clippy::missing_panics_doc)]
    pub const fn from_u128(val: u128) -> Self {
        assert!(N >= 1, "number of limbs must be greater than zero");

        let lo = val as Limb;
        let hi = (val >> 64) as Limb;

        // If there are at least 2 limbs,
        if N >= 2 {
            // we can fit `lo` and `hi`,
            let mut res = Self::ZERO;
            res.limbs[0] = lo;
            res.limbs[1] = hi;
            res
        } else if hi == Limb::ZERO {
            // or if `hi` is zero, we can fit `lo`
            let mut res = Self::ZERO;
            res.limbs[0] = lo;
            res
        } else {
            // otherwise, we panic.
            panic!("u128 is too large to fit");
        }
    }
}

/// From traits implementation for primitives.
macro_rules! impl_from_primitive {
    ($int:ty, $func_name:ident) => {
        impl<const N: usize> From<$int> for Uint<N> {
            #[inline]
            fn from(val: $int) -> Uint<N> {
                Uint::<N>::$func_name(val)
            }
        }
    };
}

impl_from_primitive!(u8, from_u8);
impl_from_primitive!(u16, from_u16);
impl_from_primitive!(u32, from_u32);
impl_from_primitive!(u64, from_u64);
impl_from_primitive!(usize, from_usize);
impl_from_primitive!(u128, from_u128);

/// Constant conversions into primitive types.
///
/// Implements conversion [`Uint`] -> `$int` for `$int` not bigger than `Limb`'s
/// max size.
macro_rules! impl_into_primitive {
    ($int:ty, $func_name:ident) => {
        impl<const N: usize> Uint<N> {
            #[doc = "Create a"]
            #[doc = stringify!($int)]
            #[doc = "integer from [`Uint`] (constant)."]
            #[doc = "# Panics"]
            #[doc = "* If [`Uint`] type is too large to fit into primitive integer."]
            #[must_use]
            #[allow(clippy::cast_possible_truncation)]
            pub const fn $func_name(self) -> $int {
                assert!(N >= 1, "number of limbs must be greater than zero");
                // Each limb besides the first one should be zero,
                const_for!((i in 1..N) {
                    // otherwise panic with overflow.
                    assert!(self.limbs[i] == 0, "Uint type is to large to fit");
                });
                // Panic if the first limb's value is bigger than maximum of resulted integer.
                assert!(
                    self.limbs[0] <= <$int>::MAX as Limb,
                    "Uint type is to large to fit"
                );

                self.limbs[0] as $int
            }
        }
    };
}

impl_into_primitive!(u8, into_u8);
impl_into_primitive!(u16, into_u16);
impl_into_primitive!(u32, into_u32);
impl_into_primitive!(u64, into_u64);
impl_into_primitive!(usize, into_usize);

impl<const N: usize> Uint<N> {
    /// Create a `u128` integer from [`Uint`] (constant).
    ///
    /// # Panics
    ///
    /// * If [`Uint`] type is too large to fit into primitive integer.
    #[must_use]
    pub const fn into_u128(self) -> u128 {
        match N {
            0 => {
                panic!("number of limbs must be greater than zero")
            }
            1 => self.limbs[0] as u128,
            _ => {
                // Each limb besides the first two should be zero,
                const_for!((i in 2..N) {
                    // otherwise panic with overflow.
                    assert!(self.limbs[i] == 0, "Uint type is to large to fit");
                });

                // Type u128 can be safely packed in two `64-bit` limbs.
                let res0 = self.limbs[0] as u128;
                let res1 = (self.limbs[1] as u128) << 64;
                res0 | res1
            }
        }
    }
}

/// From traits implementation for [`Uint`] into primitive types.
macro_rules! impl_from_uint {
    ($int:ty, $func_name:ident) => {
        impl<const N: usize> From<Uint<N>> for $int {
            #[inline]
            fn from(val: Uint<N>) -> $int {
                val.$func_name()
            }
        }
    };
}

impl_from_uint!(u8, into_u8);
impl_from_uint!(u16, into_u16);
impl_from_uint!(u32, into_u32);
impl_from_uint!(u64, into_u64);
impl_from_uint!(usize, into_usize);
impl_from_uint!(u128, into_u128);

#[cfg(feature = "ruint")]
impl<const B: usize, const L: usize> From<ruint::Uint<B, L>> for Uint<L> {
    fn from(value: ruint::Uint<B, L>) -> Self {
        // Padding ruint integer bytes.
        let mut bytes = alloc::vec![0u8; Uint::<L>::BYTES];
        let value_bytes = value.as_le_slice();
        bytes[0..value_bytes.len()].copy_from_slice(value_bytes);

        Uint::from_bytes_le(&bytes)
    }
}

#[cfg(feature = "ruint")]
impl<const B: usize, const L: usize> From<Uint<L>> for ruint::Uint<B, L> {
    fn from(value: Uint<L>) -> Self {
        // Panics if ruint::Uint size is too small.
        ruint::Uint::from_le_slice(&value.into_bytes_le())
    }
}

// ----------- Traits Impls -----------

impl<const N: usize> UpperHex for Uint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result {
        // Concatenate hex representation of limbs in reversed order without
        // allocations.
        // By the end, it will produce actual hex of `Uint`.
        for limb in self.limbs.iter().rev() {
            write!(f, "{limb:016X}")?;
        }
        Ok(())
    }
}

impl<const N: usize> Display for Uint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result {
        // Use upper hex by default.
        write!(f, "{self:X}")
    }
}

impl<const N: usize> Debug for Uint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result {
        write!(f, "{self}")
    }
}

impl<const N: usize> Ord for Uint<N> {
    #[inline]
    fn cmp(&self, rhs: &Self) -> Ordering {
        let mut result = Ordering::Equal;
        const_for_unroll6!((i in 0..N) {
            let a = &self.limbs[i];
            let b = &rhs.limbs[i];
            match a.cmp(b) {
                Ordering::Equal => {}
                order => {result = order},
            }
        });

        result
    }
}

impl<const N: usize> PartialOrd for Uint<N> {
    #[inline]
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl<const N: usize> AsMut<[u64]> for Uint<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u64] {
        &mut self.limbs
    }
}

impl<const N: usize> AsRef<[u64]> for Uint<N> {
    #[inline]
    fn as_ref(&self) -> &[u64] {
        &self.limbs
    }
}

impl<B: Borrow<Self>, const N: usize> BitXorAssign<B> for Uint<N> {
    fn bitxor_assign(&mut self, rhs: B) {
        for i in 0..N {
            self.limbs[i] ^= rhs.borrow().limbs[i];
        }
    }
}

impl<B: Borrow<Self>, const N: usize> BitXor<B> for Uint<N> {
    type Output = Self;

    fn bitxor(mut self, rhs: B) -> Self::Output {
        self ^= rhs;
        self
    }
}

impl<B: Borrow<Self>, const N: usize> BitAndAssign<B> for Uint<N> {
    fn bitand_assign(&mut self, rhs: B) {
        for i in 0..N {
            self.limbs[i] &= rhs.borrow().limbs[i];
        }
    }
}

impl<B: Borrow<Self>, const N: usize> BitAnd<B> for Uint<N> {
    type Output = Self;

    fn bitand(mut self, rhs: B) -> Self::Output {
        self &= rhs;
        self
    }
}

impl<B: Borrow<Self>, const N: usize> BitOrAssign<B> for Uint<N> {
    fn bitor_assign(&mut self, rhs: B) {
        for i in 0..N {
            self.limbs[i] |= rhs.borrow().limbs[i];
        }
    }
}

impl<B: Borrow<Self>, const N: usize> BitOr<B> for Uint<N> {
    type Output = Self;

    fn bitor(mut self, rhs: B) -> Self::Output {
        self |= rhs;
        self
    }
}

impl<const N: usize> Not for Uint<N> {
    type Output = Self;

    fn not(self) -> Self::Output {
        let mut result = Self::ZERO;
        for i in 0..N {
            result.limbs[i] = !self.limbs[i];
        }
        result
    }
}

impl<const N: usize> Shr<u32> for Uint<N> {
    type Output = Self;

    fn shr(mut self, rhs: u32) -> Self::Output {
        self >>= rhs;
        self
    }
}

impl<const N: usize> ShrAssign<u32> for Uint<N> {
    #[allow(clippy::similar_names)]
    #[allow(clippy::cast_possible_truncation)]
    fn shr_assign(&mut self, rhs: u32) {
        let shift = rhs as usize;
        let bits = Limb::BITS as usize;

        assert!(N * bits > shift, "attempt to shift right with overflow");

        // Limb shift will probably affect changes between two adjacent limbs.
        // Compute indexes of both limbs that can be changed during a single
        // iteration.
        let index2_shift = shift / bits;
        let index1_shift = index2_shift + 1;

        // The following shifts can overflow.
        // Overflow should be interpreted with zero output.
        let limb_right_shift = (shift % bits) as u32;
        let limb_left_shift = (bits - shift % bits) as u32;

        // Shift bits in limbs array in-place.
        // Start from the lowest order limb.
        for index in 0..N {
            // Take limb from index leaving 0.
            let current_limb = core::mem::take(&mut self.limbs[index]);

            if index1_shift <= index {
                let index1 = index - index1_shift;
                // Possible to copy the first part of limb with bit AND
                // operation, since the previous limbs were left zero.
                self.limbs[index1] |= current_limb
                    .checked_shl(limb_left_shift)
                    .unwrap_or_default();
            }

            if index2_shift <= index {
                let index2 = index - index2_shift;
                // Possible to copy the second part of limb with bit AND
                // operation, since the previous limbs were left zero.
                self.limbs[index2] |= current_limb
                    .checked_shr(limb_right_shift)
                    .unwrap_or_default();
            }
        }
    }
}

impl<const N: usize> Shl<u32> for Uint<N> {
    type Output = Self;

    fn shl(mut self, rhs: u32) -> Self::Output {
        self <<= rhs;
        self
    }
}

impl<const N: usize> ShlAssign<u32> for Uint<N> {
    #[allow(clippy::similar_names)]
    #[allow(clippy::cast_possible_truncation)]
    fn shl_assign(&mut self, rhs: u32) {
        let shift = rhs as usize;
        let bits = Limb::BITS as usize;

        assert!(N * bits > shift, "attempt to shift left with overflow");

        // Limb shift will probably affect changes between two adjacent limbs.
        // Compute indexes of both limbs that can be changed during a single
        // iteration.
        let index1_shift = shift / bits;
        let index2_shift = index1_shift + 1;

        // The following shifts can overflow.
        // Overflow should be interpreted with zero output.
        let limb_left_shift = (shift % bits) as u32;
        let limb_right_shift = (bits - shift % bits) as u32;

        // Shift bits in limbs array in-place.
        // Start from the highest order limb.
        for index in (0..N).rev() {
            // Take limb from index leaving 0.
            let current_limb = core::mem::take(&mut self.limbs[index]);

            let index1 = index + index1_shift;
            if index1 < N {
                // Possible to copy the first part of limb with bit AND
                // operation, since the previous limbs were left zero.
                self.limbs[index1] |= current_limb
                    .checked_shl(limb_left_shift)
                    .unwrap_or_default();
            }

            let index2 = index + index2_shift;
            if index2 < N {
                // Possible to copy the second part of limb with bit AND
                // operation, since the previous limbs were left zero.
                self.limbs[index2] |= current_limb
                    .checked_shr(limb_right_shift)
                    .unwrap_or_default();
            }
        }
    }
}

impl<const N: usize> BigInteger for Uint<N> {
    const LIMB_BITS: usize = Limb::BITS as usize;
    const MAX: Self = Self { limbs: [u64::MAX; N] };
    const NUM_LIMBS: usize = N;
    const ONE: Self = {
        let mut one = Self::ZERO;
        one.limbs[0] = 1;
        one
    };
    const ZERO: Self = Self { limbs: [0u64; N] };

    fn is_odd(&self) -> bool {
        self.is_odd()
    }

    fn is_even(&self) -> bool {
        self.is_even()
    }

    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    fn num_bits(&self) -> usize {
        self.num_bits()
    }

    fn get_bit(&self, i: usize) -> bool {
        self.get_bit(i)
    }

    fn from_bytes_le(bytes: &[u8]) -> Self {
        Self::from_le_slice(bytes)
    }

    fn into_bytes_le(self) -> Vec<u8> {
        self.limbs.iter().flat_map(|&limb| limb.to_le_bytes()).collect()
    }
}

impl<const N: usize> BitIteratorBE for Uint<N> {
    fn bit_be_iter(self) -> impl Iterator<Item = bool> {
        self.into_limbs().into_iter().rev().flat_map(Limb::bit_be_iter)
    }
}

impl BitIteratorBE for &[Limb] {
    fn bit_be_iter(self) -> impl Iterator<Item = bool> {
        self.iter().rev().copied().flat_map(Limb::bit_be_iter)
    }
}

/// Parse a number from a string in a given radix.
///
/// This implementation can be slow on big numbers and possibly fail constant
/// compilation by timeout.
///
/// I.e., convert string encoded integer `s` to base-`radix` number.
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub const fn from_str_radix<const LIMBS: usize>(
    s: &str,
    radix: u32,
) -> Uint<LIMBS> {
    let bytes = s.as_bytes();
    assert!(!bytes.is_empty(), "empty string");

    // The lowest order number is at the end of the string.
    // Begin parsing from the last index of the string.
    let mut index = bytes.len() - 1;

    let mut uint = Uint::from_u32(0);
    let mut order = Uint::from_u32(1);
    let uint_radix = Uint::from_u32(radix);

    loop {
        let digit = Uint::from_u32(parse_digit(bytes[index], radix));

        // Add a digit multiplied by order.
        uint = uint.add(&digit.mul(&order));

        // If we reached the beginning of the string, return the number.
        if index == 0 {
            return uint;
        }

        // Increase the order of magnitude.
        order = uint_radix.mul(&order);

        // Move to the next digit.
        index -= 1;
    }
}

/// Parse a number from a hex string.
///
/// This implementation performs faster than [`from_str_radix`], since it
/// assumes the radix is already `16`.
///
/// If the string number is shorter, then [`Uint`] can store, returns a [`Uint`]
/// with leading zeroes.
///
/// # Panics
///
/// * If hex encoded number is too large to fit in [`Uint`].
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub const fn from_str_hex<const LIMBS: usize>(s: &str) -> Uint<LIMBS> {
    let bytes = s.as_bytes();
    assert!(!bytes.is_empty(), "empty string");

    // The lowest order number is at the end of the string.
    // Begin parsing from the last index of the string.
    let mut index = bytes.len() - 1;

    // The lowest order limb is at the beginning of the `num` array.
    // Begin indexing from `0`.
    let mut num = [Limb::ZERO; LIMBS];
    let mut num_index = 0;

    let digit_radix = 16;
    let digit_size = 4; // Size of a hex digit in bits (2^4 = 16).
    let digits_in_limb = Limb::BITS / digit_size;

    loop {
        let digit = parse_digit(bytes[index], digit_radix) as Limb;

        let limb_index = (num_index / digits_in_limb) as usize;
        assert!(limb_index < num.len(), "hex number is too large");

        // Since a base-16 digit can be represented with the same bits, we can
        // copy these bits.
        num[limb_index] |= digit << ((num_index % digits_in_limb) * digit_size);

        // If we reached the beginning of the string, return the number.
        if index == 0 {
            return Uint::new(num);
        }

        // Move to the next digit.
        index -= 1;
        num_index += 1;
    }
}

// Try to parse a digit from utf-8 byte.
const fn parse_digit(utf8_digit: u8, digit_radix: u32) -> u32 {
    let ch = parse_utf8_byte(utf8_digit);
    match ch.to_digit(digit_radix) {
        None => {
            panic!("invalid digit");
        }
        Some(digit) => digit,
    }
}

/// Parse a single UTF-8 byte into a char.
///
/// Converts bytes to characters during compile-time string evaluation.
/// Only handles ASCII bytes (0x00-0x7F).
///
/// # Arguments
///
/// * `byte` - Byte to convert.
///
/// # Panics
///
/// * If the byte is non-ASCII (>= 0x80).
pub(crate) const fn parse_utf8_byte(byte: u8) -> char {
    match byte {
        0x00..=0x7F => byte as char,
        _ => panic!("non-ASCII character found"),
    }
}

/// This macro converts a string base-10 number to a big integer.
#[macro_export]
macro_rules! from_num {
    ($num:literal) => {
        $crate::arithmetic::uint::from_str_radix($num, 10)
    };
}

/// This macro converts a string hex number to a big integer.
#[macro_export]
macro_rules! from_hex {
    ($num:literal) => {
        $crate::arithmetic::uint::from_str_hex($num)
    };
}

/// Integer that uses twice more limbs than `Uint` for the same `N` parameter.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Zeroize)]
pub struct WideUint<const N: usize> {
    low: Uint<N>,
    high: Uint<N>,
}

impl<const N: usize> WideUint<N> {
    /// Construct new [`WideUint`] from `low` and `high` parts.
    #[must_use]
    pub const fn new(low: Uint<N>, high: Uint<N>) -> Self {
        Self { low, high }
    }

    /// Compute the remainder of division `self` by `rhs` (constant).
    ///
    /// Basic division algorithm based on [wiki].
    /// Fine to be used for constant evaluation, but slow in runtime.
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Division_algorithm
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub const fn rem(&self, rhs: &Uint<N>) -> Uint<N> {
        assert!(!rhs.is_zero(), "should not divide by zero");

        let mut remainder = Uint::<N>::ZERO;
        let num_bits = self.num_bits();

        // Start from the last bit.
        const_rev_for!((index in 0..num_bits) {
            // Shift the remainder to the left by 1,
            let (result, carry) = remainder.checked_mul2();
            remainder = result;

            // and set the first bit to remainder from the dividend.
            remainder.limbs[0] |= self.get_bit(index) as Limb;

            // If the remainder overflows, subtract the divisor.
            if remainder.ge(rhs) || carry {
                (remainder, _) = remainder.checked_sub(rhs);
            }
        });

        remainder
    }

    /// Find the number of bits in the binary decomposition of `self`.
    ///
    /// One bit is necessary to encode zero.
    #[must_use]
    pub const fn num_bits(&self) -> usize {
        if self.high.is_zero() {
            self.low.num_bits()
        } else {
            self.high.num_bits() + Uint::<N>::BITS
        }
    }

    /// Compute the `i`-th bit of `self`.
    #[must_use]
    pub const fn get_bit(&self, i: usize) -> bool {
        if i >= Uint::<N>::BITS {
            self.high.get_bit(i - Uint::<N>::BITS)
        } else {
            self.low.get_bit(i)
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::*;
    use crate::{
        arithmetic::{
            uint::{
                from_str_hex, from_str_radix, Uint, WideUint, U256, U512, U64,
            },
            BigInteger, Limb,
        },
        bits::BitIteratorBE,
    };

    macro_rules! test_uxxx_default {
        ($($type:ident),* $(,)?) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<default_ $type:lower>]() {
                        let uint = $type::default();
                        assert_eq!(uint, $type::ZERO);
                    }
                }
            )*
        };
    }

    // Usage: Generate tests for all UXXX types
    test_uxxx_default! {
        U64,
        U128,
        U192,
        U256,
        U384,
        U448,
        U512,
        U576,
        U640,
        U704,
        U768,
        U832,
    }

    #[test]
    fn convert_from_str_radix() {
        let uint_from_base10: Uint<4> = from_str_radix(
            "28948022309329048855892746252171976963363056481941647379679742748393362948097",
            10,
        );
        #[allow(clippy::unreadable_literal)]
        let expected = Uint::<4>::new([
            10108024940646105089u64,
            2469829653919213789u64,
            0u64,
            4611686018427387904u64,
        ]);
        assert_eq!(uint_from_base10, expected);

        let uint_from_base10: Uint<1> =
            from_str_radix("18446744069414584321", 10);
        let uint_from_binary: Uint<1> = from_str_radix(
            "1111111111111111111111111111111100000000000000000000000000000001",
            2,
        );
        assert_eq!(uint_from_base10, uint_from_binary);
    }

    #[test]
    fn convert_from_str_hex() {
        // Test different implementations of hex parsing on random hex inputs.
        proptest!(|(hex in "[0-9a-fA-F]{1,64}")| {
            let uint_from_hex: Uint<4> = from_str_hex(&hex);
            let expected: Uint<4> = from_str_radix(&hex, 16);
            prop_assert_eq!(uint_from_hex, expected);
        });
    }

    #[test]
    #[should_panic = "hex number is too large"]
    fn from_str_hex_should_panic_on_overflow() {
        let _ = from_str_hex::<4>(
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0",
        );
    }

    #[test]
    fn parse_and_display_hex() {
        // Test parsing from upper hex against displaying in upper hex.
        proptest!(|(upper_hex in "[0-9A-F]{64}")| {
            let uint_from_hex: Uint<4> = from_str_hex(&upper_hex);
            let hex_from_uint = format!("{uint_from_hex:X}");
            prop_assert_eq!(hex_from_uint, upper_hex);
        });
    }

    #[test]
    fn uint_bit_iterator_be() {
        let words: [Limb; 4] = [0b1100, 0, 0, 0];
        let num = Uint::<4>::new(words);
        let bits: Vec<bool> = num.bit_be_trimmed_iter().collect();

        assert_eq!(bits.len(), 4);
        assert_eq!(bits, vec![true, true, false, false]);
    }

    #[test]
    fn num_bits() {
        let words: [Limb; 4] = [0b1100, 0, 0, 0];
        let num = Uint::<4>::new(words);
        assert_eq!(num.num_bits(), 4);

        let words: [Limb; 4] = [0, 0b1100, 0, 0];
        let num = Uint::<4>::new(words);
        assert_eq!(num.num_bits(), 64 + 4);

        let words: [Limb; 4] = [0b11, 0b11, 0b11, 0b11];
        let num = Uint::<4>::new(words);
        assert_eq!(num.num_bits(), 64 + 64 + 64 + 2);
    }

    #[test]
    fn rem() {
        let dividend = from_num!("43129923721897334698312931");
        let divisor = from_num!("375923422");
        let result =
            WideUint::<4>::new(dividend, Uint::<4>::ZERO).rem(&divisor);
        assert_eq!(result, from_num!("216456157"));
    }

    #[test]
    #[should_panic = "should not divide by zero"]
    fn rem_zero() {
        let zero = Uint::<4>::ZERO;
        let divisor = from_num!("375923422");
        let result = WideUint::<4>::new(zero, zero).rem(&divisor);
        assert_eq!(result, zero);

        let dividend = from_num!("43129923721897334698312931");
        let divisor = zero;
        let _ = WideUint::<4>::new(dividend, zero).rem(&divisor);
    }

    #[test]
    fn ge_le_gt_lt_eq_ne() {
        let a: Uint<6> = Uint::new([0, 0, 0, 0, 0, 5]);
        let b: Uint<6> = Uint::new([4, 0, 0, 0, 0, 0]);
        assert!(a.ge(&b));
        assert!(a.gt(&b));
        assert!(!a.le(&b));
        assert!(!a.lt(&b));
        assert!(!a.eq(&b));
        assert!(a.ne(&b));

        let a: Uint<6> = Uint::new([0, 0, 0, 0, 0, 5]);
        let b: Uint<6> = Uint::new([0, 0, 0, 0, 0, 6]);
        assert!(!a.ge(&b));
        assert!(!a.gt(&b));
        assert!(a.le(&b));
        assert!(a.lt(&b));
        assert!(!a.eq(&b));
        assert!(a.ne(&b));

        let a: Uint<4> = Uint::new([0, 0, 1, 2]);
        let b: Uint<4> = Uint::new([0, 0, 1, 2]);
        assert!(a.ge(&b));
        assert!(!a.gt(&b));
        assert!(a.le(&b));
        assert!(!a.lt(&b));
        assert!(a.eq(&b));
        assert!(!a.ne(&b));
    }

    #[test]
    fn checked_mul2_assign() {
        let mut a: Uint<6> = Uint::new([0, 0, 0, 0, 0, 5]);

        let overflowed = a.checked_mul2_assign();
        assert!(!overflowed);

        let expected: Uint<6> = Uint::new([0, 0, 0, 0, 0, 10]);
        assert_eq!(expected, a);

        let mut a: Uint<1> = Uint::new([u64::MAX]);

        let overflowed = a.checked_mul2_assign();
        assert!(overflowed);

        let expected: Uint<1> = Uint::new([u64::MAX - 1]);
        assert_eq!(expected, a);
    }

    #[test]
    fn shl() {
        // The first limb is the lowest order part of the number.
        let num = Uint::<4>::new([0b1100000000, 0, 0, 0]);

        let expected = Uint::<4>::new([0, 0b11000000, 0, 0]);
        assert_eq!(num << 62, expected);

        let expected = Uint::<4>::new([0, 0, 0b110000, 0]);
        assert_eq!(num << (60 + 64), expected);

        let expected = Uint::<4>::new([0, 0, 0, 0b1100]);
        assert_eq!(num << (58 + 64 + 64), expected);

        // edge case to make shift the number into all zeroes
        let expected = Uint::<4>::new([0, 0, 0, 0]);
        assert_eq!(num << (56 + 64 + 64 + 64), expected);
    }

    #[test]
    #[should_panic = "attempt to shift left with overflow"]
    fn shl_overflow_should_panic() {
        let num = Uint::<4>::ONE;
        let _ = num << (64 * 4);
    }

    #[test]
    fn shr() {
        // The last limb is the highest order part of the number.
        let num = Uint::<4>::new([0, 0, 0, 0b11]);

        let expected = Uint::<4>::new([0, 0, 0b1100, 0]);
        assert_eq!(num >> 62, expected);

        let expected = Uint::<4>::new([0, 0b110000, 0, 0]);
        assert_eq!(num >> (60 + 64), expected);

        let expected = Uint::<4>::new([0b11000000, 0, 0, 0]);
        assert_eq!(num >> (58 + 64 + 64), expected);

        // edge case to make shift the number into all zeroes
        let expected = Uint::<4>::new([0, 0, 0, 0]);
        assert_eq!(num >> (2 + 64 + 64 + 64), expected);
    }

    #[test]
    #[should_panic = "attempt to shift right with overflow"]
    fn shr_overflow_should_panic() {
        let num = Uint::<4>::ONE;
        let _ = num >> (64 * 4);
    }

    #[test]
    fn shr_shl_edge_case() {
        let num = Uint::<4>::ONE;
        assert_eq!(num >> 0, num);
        assert_eq!(num << 0, num);

        let num = Uint::<4>::new([
            0xffffffffffffffff,
            0xffffffffffffffff,
            0,
            0xffffffffffffffff,
        ]);

        assert_eq!(
            num >> 64,
            Uint::<4>::new([0xffffffffffffffff, 0, 0xffffffffffffffff, 0])
        );

        assert_eq!(
            num << 64,
            Uint::<4>::new([0, 0xffffffffffffffff, 0xffffffffffffffff, 0])
        );
    }

    #[test]
    fn test_process_single_element_masks_correctly() {
        let low_part_bits = 248;
        let low_part_mask: U256 = from_str_hex(
            "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        let element: U256 = from_str_hex(
            "01ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        let high_part = element >> low_part_bits;
        let low_part = element & low_part_mask;
        assert_eq!(high_part, U256::ONE);
        assert_eq!(low_part, low_part_mask);
    }

    #[cfg(feature = "ruint")]
    mod ruint_conversion_test {
        use super::*;

        /// This macro generates property-based tests for bidirectional
        /// conversions between [`ruint::Uint`] and [`Uint`] types.
        ///
        /// Each test verifies that round-trip conversions preserve the original
        /// value: `ruint::Uint -> Uint -> ruint::Uint` should equal the
        /// original value.
        ///
        /// The number of limbs is automatically calculated using
        /// `usize::div_ceil(bits, Limb::BITS)`.
        macro_rules! test_ruint_conversion {
            ($test_name:ident, $uint_type:ident, $bits:expr) => {
                #[test]
                fn $test_name() {
                    proptest!(|(value: ruint::Uint<$bits, { usize::div_ceil($bits, $crate::arithmetic::Limb::BITS as usize) }>)| {
                        let uint_from_ruint: crate::arithmetic::uint::$uint_type = value.into();
                        let expected: ruint::Uint<$bits, { usize::div_ceil($bits, $crate::arithmetic::Limb::BITS as usize) }> = uint_from_ruint.into();
                        prop_assert_eq!(value, expected);
                    });
                }
            };
        }

        test_ruint_conversion!(ruint_u64, U64, 64);
        test_ruint_conversion!(ruint_u128, U128, 128);
        test_ruint_conversion!(ruint_u256, U256, 256);
    }

    mod primitive_conversion {
        use super::*;

        macro_rules! test_conversion {
            ($($type:ty),*) => {
                $(
                    paste::paste! {
                        #[test]
                        fn $type() {
                            proptest!(|(expected_primitive_num: $type)| {
                                let num: U256 = expected_primitive_num.into();
                                let primitive_num: $type = num.into();
                                assert_eq!(expected_primitive_num, primitive_num);
                            });
                        }
                    }
                )*
            };
        }

        test_conversion!(u8, u16, u32, u64, u128, usize);
    }

    #[test]
    fn is_zero_uint() {
        let x = Uint::<4>::new([0, 0, 0, 0]);
        assert!(x.is_zero());

        for x in [
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1],
            [u64::MAX, u64::MAX, u64::MAX, u64::MAX],
        ]
        .map(Uint::<4>::new)
        {
            assert!(!x.is_zero());
        }
    }

    #[test]
    fn get_bit_for_uint() {
        let a: Uint<4> = Uint::new([2, 0, 0, 0]);

        let i_of_1 = 1;

        assert!(a.get_bit(i_of_1));

        assert!(a.get_bit(i_of_1));

        let i_too_large = 256;
        assert!(!a.get_bit(i_too_large));

        assert!(!a.get_bit(i_too_large));
    }

    #[test]
    fn num_bits_for_wide_uint() {
        let x = WideUint::<4>::new(
            Uint::new([0, 0, 0, 0]),
            Uint::new([0, 0, 0, 0]),
        );
        assert_eq!(1, x.num_bits());

        let x = WideUint::<4>::new(
            Uint::new([1, 0, 0, 0]),
            Uint::new([0, 0, 0, 0]),
        );
        assert_eq!(1, x.num_bits());

        let x = WideUint::<4>::new(
            Uint::new([2, 0, 0, 0]),
            Uint::new([0, 0, 0, 0]),
        );
        assert_eq!(2, x.num_bits());

        let x = WideUint::<4>::new(
            Uint::new([2, 0, 0, 0]),
            Uint::new([1, 0, 0, 0]),
        );
        assert_eq!(257, x.num_bits());

        let x = WideUint::<4>::new(
            Uint::new([1, 0, 0, 0]),
            Uint::new([0, 0, 0, 0x8000_0000_0000_0000]),
        );
        assert_eq!(512, x.num_bits());

        let x = WideUint::<4>::new(
            Uint::new([0, 0, 0, 0]),
            Uint::new([0, 0, 0, 0x8000_0000_0000_0000]),
        );
        assert_eq!(512, x.num_bits());
    }

    #[test]
    fn get_bit_for_wide_uint() {
        let a: WideUint<4> = WideUint::new(
            Uint::new([2, 0, 0, 0]),
            Uint::new([0, 0, 0, 0x8000_0000_0000_0000]),
        );

        let i_low = 1;

        assert!(a.get_bit(i_low));

        let i_high = 511;
        assert!(a.get_bit(i_high));

        let i_too_large = 512;
        assert!(!a.get_bit(i_too_large));
    }

    #[test]
    fn from_u128_with_one_limb() {
        let prim: u128 = u64::MAX.into();
        let target = Uint::<1>::from_u128(prim);
        assert_eq!(u64::MAX, target.limbs[0]);
    }

    #[test]
    #[should_panic = "u128 is too large to fit"]
    fn from_u128_panics_on_u128_overflow() {
        let prim: u128 = u128::MAX;
        _ = Uint::<1>::from_u128(prim);
    }

    #[test]
    #[should_panic = "bytes are not the expected size"]
    fn from_le_slice_panics_on_unexpected_byte_length() {
        let bytes = &[];
        _ = Uint::<1>::from_le_slice(bytes);
    }

    #[test]
    #[should_panic = "non-ASCII character found"]
    fn parse_utf8_byte_panics_for_invalid_ascii_byte() {
        _ = parse_utf8_byte(0x80);
    }

    #[test]
    #[should_panic = "invalid digit"]
    fn parse_digit_panics_for_invalid_digit() {
        _ = parse_digit(0xf, 10);
    }

    #[test]
    fn display_uint() {
        let x = Uint::<4>::new([0xf, 1, 0, u64::MAX]);
        assert_eq!(
            "FFFFFFFFFFFFFFFF00000000000000000000000000000001000000000000000F",
            format!("{x}")
        );
    }

    #[test]
    fn debug_uint() {
        let x = Uint::<4>::new([0xf, 1, 0, u64::MAX]);
        assert_eq!(
            "FFFFFFFFFFFFFFFF00000000000000000000000000000001000000000000000F",
            format!("{x:?}")
        );
    }

    #[test]
    #[should_panic = "Uint type is to large to fit"]
    fn into_primitive_panics_on_too_large_uint() {
        let x = Uint::<1>::new([u64::MAX]);
        _ = x.into_u32();
    }

    #[test]
    fn as_ref_for_uint() {
        let limbs = [0xf, 1, 0, u64::MAX];
        let mut x = Uint::<4>::new(limbs);
        assert_eq!(limbs, x.as_mut());
        assert_eq!(limbs, x.as_ref());
    }

    mod bit_be_iter {
        use super::*;

        #[test]
        fn limb_slice_bit_iter_empty() {
            let empty_slice: &[Limb] = &[];
            let bits: Vec<bool> = empty_slice.bit_be_iter().collect();
            assert_eq!(bits, vec![]);
        }

        #[test]
        fn limb_slice_bit_iter_single_limb() {
            // Test with a single limb containing a simple pattern
            let limbs: &[Limb] = &[0b1010_1100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000];
            let bits: Vec<bool> = limbs.bit_be_iter().collect();

            // Should have exactly 64 bits
            assert_eq!(bits.len(), 64);

            // Check the first few bits match our pattern (1010_1100...)
            assert!(bits[0]); // MSB
            assert!(!bits[1]);
            assert!(bits[2]);
            assert!(!bits[3]);
            assert!(bits[4]);
            assert!(bits[5]);
            assert!(!bits[6]);
            assert!(!bits[7]);

            // Rest should be false (zeros)
            for i in 8..64 {
                assert!(!bits[i], "Bit {i} should be false");
            }
        }

        #[test]
        fn limb_slice_bit_iter_multiple_limbs() {
            // Test with multiple limbs - order should be reversed in iteration
            let limbs: &[Limb] = &[
                0x0000_0000_0000_000F, // Low limb (last in iteration)
                0xF000_0000_0000_0000, // High limb (first in iteration)
            ];

            let bits: Vec<bool> = limbs.bit_be_iter().collect();
            assert_eq!(bits.len(), 128); // 2 limbs × 64 bits each

            // First 4 bits should be from the high limb (0xF000...)
            assert!(bits[0]); // F = 1111
            assert!(bits[1]);
            assert!(bits[2]);
            assert!(bits[3]);

            // Next 60 bits from high limb should be false
            for i in 4..64 {
                assert!(!bits[i], "High limb bit {i} should be false");
            }

            // First 60 bits from low limb should be false
            for i in 64..124 {
                assert!(!bits[i], "Low limb bit {} should be false", i - 64);
            }

            // Last 4 bits should be from the low limb (0x...F)
            assert!(bits[124]); // F = 1111
            assert!(bits[125]);
            assert!(bits[126]);
            assert!(bits[127]);
        }

        #[test]
        fn limb_slice_bit_iter_all_zeros() {
            let limbs: &[Limb] = &[0, 0, 0];
            let bits: Vec<bool> = limbs.bit_be_iter().collect();

            assert_eq!(bits.len(), 192); // 3 limbs × 64 bits
            assert!(bits.iter().all(|&b| !b), "All bits should be false");
        }

        #[test]
        fn limb_slice_bit_iter_all_ones() {
            let limbs: &[Limb] = &[Limb::MAX, Limb::MAX];
            let bits: Vec<bool> = limbs.bit_be_iter().collect();

            assert_eq!(bits.len(), 128); // 2 limbs × 64 bits
            assert!(bits.iter().all(|&b| b), "All bits should be true");
        }

        #[test]
        fn limb_slice_bit_iter_ordering() {
            // Test that limbs are processed in reverse order (big-endian)
            let limbs: &[Limb] = &[
                0x0000_0000_0000_0001, // Index 0 - should be processed last
                0x8000_0000_0000_0000, // Index 1 - should be processed first
            ];

            let bits: Vec<bool> = limbs.bit_be_iter().collect();

            // First bit should be the MSB of the last limb (index 1)
            assert!(bits[0]);
            // All other bits from limb[1] should be false
            for i in 1..64 {
                assert!(!bits[i]);
            }

            // All bits from limb[0] except the last should be false
            for i in 64..127 {
                assert!(!bits[i]);
            }
            // Last bit should be the LSB of the first limb (index 0)
            assert!(bits[127]);
        }
    }

    #[test]
    fn wrapping_add() {
        // Test non-wrapping addition
        let a = Uint::<4>::from_u128(100);
        let b = Uint::<4>::from_u128(50);
        let expected = Uint::<4>::from_u128(150);

        let result = a.wrapping_add(&b);
        assert_eq!(result, expected, "Non-wrapping addition failed");

        // Verify this addition doesn't overflow by checking with adc
        let (sum, carry) = a.adc(&b, false);
        assert_eq!(result, sum);
        assert!(!carry, "Non-wrapping addition should not produce carry");

        // Test wrapping addition - use maximum value to guarantee overflow
        let max_val = Uint::<2>::MAX; // Using smaller type for easier overflow
        let one = Uint::<2>::from_u64(1);

        // MAX + 1 should wrap to 0
        let wrapped_result = max_val.wrapping_add(&one);
        let expected_wrapped = Uint::<2>::ZERO;
        assert_eq!(
            wrapped_result, expected_wrapped,
            "Wrapping addition MAX + 1 should equal 0"
        );

        // Verify this addition does overflow
        let (sum_with_carry, carry) = max_val.adc(&one, false);
        assert_eq!(wrapped_result, sum_with_carry);
        assert!(carry, "Addition of MAX + 1 should produce carry (overflow)");

        // Test another wrapping case: MAX + MAX should equal MAX - 1 (due to
        // wrap)
        let wrapped_max_plus_max = max_val.wrapping_add(&max_val);
        let expected_max_plus_max = Uint::<2>::MAX.wrapping_sub(&one); // MAX - 1
        assert_eq!(
            wrapped_max_plus_max, expected_max_plus_max,
            "MAX + MAX should wrap to MAX - 1"
        );

        // Verify this overflows
        let (_, carry_max_plus_max) = max_val.adc(&max_val, false);
        assert!(carry_max_plus_max, "MAX + MAX should produce carry");

        // Test large number wrapping with U256
        let large1: Uint<4> = from_str_hex(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        );
        let large2: Uint<4> = from_str_hex(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        );
        let wrapped_large = large1.wrapping_add(&large2);
        let expected_large: Uint<4> = from_str_hex(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE",
        );
        assert_eq!(
            wrapped_large, expected_large,
            "Large number wrapping addition failed"
        );

        // Verify the large addition overflows
        let (_, large_carry) = large1.adc(&large2, false);
        assert!(large_carry, "Large number addition should overflow");

        // Test adding zero (should never wrap)
        let val = Uint::<4>::from_u128(12345);
        let zero = Uint::<4>::ZERO;
        let result = val.wrapping_add(&zero);
        assert_eq!(result, val, "Adding zero should return original value");

        let (sum, carry) = val.adc(&zero, false);
        assert_eq!(result, sum);
        assert!(!carry, "Adding zero should never produce carry");

        // Test zero + zero
        let zero_plus_zero = zero.wrapping_add(&zero);
        assert_eq!(zero_plus_zero, zero, "Zero plus zero should equal zero");

        // Test maximum value + zero (should not wrap)
        let max_plus_zero = Uint::<4>::MAX.wrapping_add(&zero);
        assert_eq!(max_plus_zero, Uint::<4>::MAX, "MAX + 0 should equal MAX");

        let (max_sum, max_carry) = Uint::<4>::MAX.adc(&zero, false);
        assert_eq!(max_plus_zero, max_sum);
        assert!(!max_carry, "MAX + 0 should not produce carry");
    }

    #[test]
    fn bitxor_assign() {
        let mut a: Uint<4> = from_str_hex(
            "F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0",
        );
        let b: Uint<4> = from_str_hex(
            "0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F",
        );
        let expected: Uint<4> = from_str_hex(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        );

        a ^= b;
        assert_eq!(a, expected, "BitXorAssign failed");

        // Test XOR with self (should result in zero)
        let mut c = Uint::<2>::from_u128(0x123456789ABCDEF0);
        let original = c;
        c ^= original;
        assert_eq!(c, Uint::<2>::ZERO, "XOR with self should be zero");

        // Test XOR with zero (should be unchanged)
        let mut d = Uint::<2>::from_u128(0xDEADBEEFCAFEBABE);
        let original_d = d;
        d ^= Uint::<2>::ZERO;
        assert_eq!(d, original_d, "XOR with zero should be unchanged");
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn bitxor() {
        let a = Uint::<2>::from_u128(0xAAAAAAAAAAAAAAAA5555555555555555);
        let b = Uint::<2>::from_u128(0x5555555555555555AAAAAAAAAAAAAAAA);
        let expected: Uint<2> =
            from_str_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

        let result = a ^ b;
        assert_eq!(result, expected, "BitXor failed");

        // Test that original values are unchanged
        assert_eq!(a, Uint::<2>::from_u128(0xAAAAAAAAAAAAAAAA5555555555555555));
        assert_eq!(b, Uint::<2>::from_u128(0x5555555555555555AAAAAAAAAAAAAAAA));

        // Test XOR with borrowed value
        let result2 = a ^ &b;
        assert_eq!(result2, expected, "BitXor with borrowed value failed");
    }

    #[test]
    fn bitor_assign() {
        let mut a: Uint<2> = from_str_hex("F0F0F0F0F0F0F0F00F0F0F0F0F0F0F0F");
        let b: Uint<2> = from_str_hex("0F0F0F0F0F0F0F0FF0F0F0F0F0F0F0F0");
        let expected: Uint<2> =
            from_str_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

        a |= b;
        assert_eq!(a, expected, "BitOrAssign failed");

        // Test OR with zero (should be unchanged)
        let mut c = Uint::<2>::from_u128(0x123456789ABCDEF0);
        let original = c;
        c |= Uint::<2>::ZERO;
        assert_eq!(c, original, "OR with zero should be unchanged");

        // Test OR with self (should be unchanged)
        let mut d = Uint::<2>::from_u128(0xDEADBEEFCAFEBABE);
        let original_d = d;
        d |= original_d;
        assert_eq!(d, original_d, "OR with self should be unchanged");
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn bitor() {
        let a: Uint<2> = from_str_hex("FF00FF00FF00FF0000FF00FF00FF00FF");
        let b: Uint<2> = from_str_hex("00FF00FF00FF00FFFF00FF00FF00FF00");
        let expected: Uint<2> =
            from_str_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

        let result = a | b;
        assert_eq!(result, expected, "BitOr failed");

        // Test that original values are unchanged
        assert_eq!(a, from_str_hex::<2>("FF00FF00FF00FF0000FF00FF00FF00FF"));
        assert_eq!(b, from_str_hex::<2>("00FF00FF00FF00FFFF00FF00FF00FF00"));

        // Test OR with borrowed value
        let result2 = a | &b;
        assert_eq!(result2, expected, "BitOr with borrowed value failed");

        // Test OR with MAX (should always be MAX)
        let max_result = a | Uint::<2>::MAX;
        assert_eq!(max_result, Uint::<2>::MAX, "OR with MAX should be MAX");
    }

    #[test]
    fn not() {
        // Test NOT operation
        let a: Uint<2> = from_str_hex("F0F0F0F0F0F0F0F00F0F0F0F0F0F0F0F");
        let expected: Uint<2> =
            from_str_hex("0F0F0F0F0F0F0F0FF0F0F0F0F0F0F0F0");

        let result = !a;
        assert_eq!(result, expected, "Not operation failed");

        // Test that original value is unchanged
        assert_eq!(a, from_str_hex::<2>("F0F0F0F0F0F0F0F00F0F0F0F0F0F0F0F"));

        // Test NOT of zero should be MAX
        let not_zero = !Uint::<2>::ZERO;
        assert_eq!(not_zero, Uint::<2>::MAX, "NOT of zero should be MAX");

        // Test NOT of MAX should be zero
        let not_max = !Uint::<2>::MAX;
        assert_eq!(not_max, Uint::<2>::ZERO, "NOT of MAX should be zero");

        // Test double NOT should return original
        let double_not = !!a;
        assert_eq!(double_not, a, "Double NOT should return original");
    }

    #[test]
    fn bitwise_operations_single_limb() {
        // Test with single limb types
        let a = Uint::<1>::from_u64(0xAAAAAAAAAAAAAAAA);
        let b = Uint::<1>::from_u64(0x5555555555555555);

        // XOR
        let xor_result = a ^ b;
        assert_eq!(xor_result, Uint::<1>::from_u64(0xFFFFFFFFFFFFFFFF));

        // OR
        let or_result = a | b;
        assert_eq!(or_result, Uint::<1>::from_u64(0xFFFFFFFFFFFFFFFF));

        // NOT
        let not_a = !a;
        assert_eq!(not_a, Uint::<1>::from_u64(0x5555555555555555));
    }

    #[test]
    fn bitwise_assign_operations() {
        let mut a: Uint<2> = from_str_hex("AAAAAAAAAAAAAAAA5555555555555555");
        let b: Uint<2> = from_str_hex("5555555555555555AAAAAAAAAAAAAAAA");
        let original_a = a;
        let original_b = b;

        // Test XOR assign
        a ^= &b; // Test with borrowed value
        let expected = original_a ^ original_b;
        assert_eq!(a, expected, "XorAssign with borrowed value failed");

        // Reset and test OR assign
        a = original_a;
        a |= b; // Test with owned value
        let expected = original_a | original_b;
        assert_eq!(a, expected, "OrAssign with owned value failed");

        // Verify b is unchanged (it was moved but we test the concept)
        assert_eq!(
            original_b,
            from_str_hex::<2>("5555555555555555AAAAAAAAAAAAAAAA")
        );
    }

    #[test]
    fn checked_add() {
        // Test non-overflowing addition
        let a = Uint::<2>::from_u128(100);
        let b = Uint::<2>::from_u128(50);
        let (result, overflow) = a.checked_add(&b);

        assert_eq!(result, Uint::<2>::from_u128(150));
        assert!(!overflow, "Addition should not overflow");

        // Test overflowing addition
        let max = Uint::<2>::MAX;
        let one = Uint::<2>::from_u64(1);
        let (result_overflow, overflow_flag) = max.checked_add(&one);

        assert_eq!(result_overflow, Uint::<2>::ZERO);
        assert!(overflow_flag, "MAX + 1 should overflow");

        // Test adding zero
        let val = Uint::<2>::from_u128(12345);
        let zero = Uint::<2>::ZERO;
        let (result_zero, overflow_zero) = val.checked_add(&zero);

        assert_eq!(result_zero, val);
        assert!(!overflow_zero, "Adding zero should not overflow");

        // Test large overflow case
        let large1 = Uint::<6>::new([
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ]);
        let large2 = Uint::<6>::new([
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ]);
        let (result_large, overflow_large) = large1.checked_add(&large2);
        let expected_large = Uint::<6>::new([
            u64::MAX - 1,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ]);

        assert_eq!(result_large, expected_large);
        assert!(overflow_large, "Large addition should overflow");
    }

    #[test]
    fn checked_add_assign() {
        // Test non-overflowing addition
        let mut a = Uint::<2>::from_u128(100);
        let b = Uint::<2>::from_u128(50);
        let overflow = a.checked_add_assign(&b);

        assert_eq!(a, Uint::<2>::from_u128(150));
        assert!(!overflow, "Addition should not overflow");

        // Test overflowing addition
        let mut max = Uint::<2>::MAX;
        let one = Uint::<2>::from_u64(1);
        let overflow_flag = max.checked_add_assign(&one);

        assert_eq!(max, Uint::<2>::ZERO);
        assert!(overflow_flag, "MAX + 1 should overflow");

        // Test adding zero
        let mut val = Uint::<2>::from_u128(12345);
        let original = val;
        let zero = Uint::<2>::ZERO;
        let overflow_zero = val.checked_add_assign(&zero);

        assert_eq!(val, original);
        assert!(!overflow_zero, "Adding zero should not overflow");

        // Test multiple additions with carry propagation
        let mut multi = Uint::<6>::new([
            u64::MAX - 1,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ]);
        let one = Uint::<6>::from_u64(1);
        let overflow1 = multi.checked_add_assign(&one);
        assert_eq!(multi, Uint::<6>::MAX);
        assert!(!overflow1, "Should not overflow yet");

        let overflow2 = multi.checked_add_assign(&one);
        assert_eq!(multi, Uint::<6>::ZERO);
        assert!(overflow2, "Should overflow now");
    }

    #[test]
    fn checked_sub_assign() {
        // Test non-underflowing subtraction
        let mut a = Uint::<2>::from_u128(150);
        let b = Uint::<2>::from_u128(50);
        let underflow = a.checked_sub_assign(&b);

        assert_eq!(a, Uint::<2>::from_u128(100));
        assert!(!underflow, "Subtraction should not underflow");

        // Test underflowing subtraction
        let mut zero = Uint::<2>::ZERO;
        let one = Uint::<2>::from_u64(1);
        let underflow_flag = zero.checked_sub_assign(&one);

        assert_eq!(zero, Uint::<2>::MAX); // Wraps around to MAX
        assert!(underflow_flag, "0 - 1 should underflow");

        // Test subtracting zero
        let mut val = Uint::<2>::from_u128(12345);
        let original = val;
        let zero_val = Uint::<2>::ZERO;
        let underflow_zero = val.checked_sub_assign(&zero_val);

        assert_eq!(val, original);
        assert!(!underflow_zero, "Subtracting zero should not underflow");

        // Test subtracting from self (should result in zero)
        let mut self_sub = Uint::<2>::from_u128(54321);
        let original_self = self_sub;
        let underflow_self = self_sub.checked_sub_assign(&original_self);

        assert_eq!(self_sub, Uint::<2>::ZERO);
        assert!(!underflow_self, "Subtracting from self should not underflow");

        // Test large underflow case
        let mut small = Uint::<6>::from_u64(1);
        let large = Uint::<6>::new([
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        ]);
        let underflow_large = small.checked_sub_assign(&large);

        assert_eq!(small, Uint::<6>::from_u64(2)); // 1 - MAX = 2 (wrapping)
        assert!(underflow_large, "Small - large should underflow");
    }

    #[test]
    fn consistency_between_methods() {
        // Test that checked_add and checked_add_assign produce same results
        let a = Uint::<2>::from_u128(12345);
        let b = Uint::<2>::from_u128(67890);

        // Test non-overflow case
        let (result_const, overflow_const) = a.checked_add(&b);
        let mut a_mut = a;
        let overflow_assign = a_mut.checked_add_assign(&b);

        assert_eq!(result_const, a_mut, "Results should be equal");
        assert_eq!(
            overflow_const, overflow_assign,
            "Overflow flags should be equal"
        );

        // Test overflow case
        let max = Uint::<2>::MAX;
        let one = Uint::<2>::from_u64(1);

        let (result_overflow, overflow_flag1) = max.checked_add(&one);
        let mut max_mut = max;
        let overflow_flag2 = max_mut.checked_add_assign(&one);

        assert_eq!(
            result_overflow, max_mut,
            "Overflow results should be equal"
        );
        assert_eq!(
            overflow_flag1, overflow_flag2,
            "Overflow flags should be equal"
        );
    }

    #[test]
    fn edge_cases() {
        // Test with single limb
        let mut a = Uint::<1>::from_u64(u64::MAX - 1);
        let one = Uint::<1>::from_u64(1);

        let overflow1 = a.checked_add_assign(&one);
        assert_eq!(a, Uint::<1>::from_u64(u64::MAX));
        assert!(!overflow1, "Should not overflow yet");

        let overflow2 = a.checked_add_assign(&one);
        assert_eq!(a, Uint::<1>::ZERO);
        assert!(overflow2, "Should overflow now");

        // Test subtraction edge case
        let mut b = Uint::<1>::from_u64(1);
        let two = Uint::<1>::from_u64(2);
        let underflow = b.checked_sub_assign(&two);

        assert_eq!(b, Uint::<1>::from_u64(u64::MAX)); // Wraps to MAX
        assert!(underflow, "Should underflow");
    }

    #[test]
    fn edge_case_u64_to_u128() {
        let uint_origin: U64 = from_str_hex("ff");
        let tmp: u128 = uint_origin.into();
        assert_eq!(tmp, 0xff);
    }

    #[cfg(feature = "ruint")]
    #[test]
    fn test_ruint_to_uint_conversion_unexpected_panic() {
        let ruint_origin: ruint::Uint<200, 4> = ruint::Uint::from(42);
        // 256 > 200, Should success
        let _uint_from_ruint: U256 = ruint_origin.into();
    }

    #[test]
    fn from_uint() {
        // Check that conversion between integers with different bit size works.
        proptest!(|(limbs: [u64; 4])|{
            let expected_uint = U256::new(limbs);
            let wide_uint = U512::from_uint(expected_uint);
            let uint = U256::from_uint(wide_uint);

            assert_eq!(uint, expected_uint);
        });
    }
}
