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
    ct_for, ct_for_unroll6,
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

    /// Returns true if this number is odd (constant).
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn ct_is_odd(&self) -> bool {
        self.limbs[0] & 1 == 1
    }

    /// Returns true if this number is even (constant).
    #[doc(hidden)]
    #[inline]
    #[must_use]
    pub const fn ct_is_even(&self) -> bool {
        self.limbs[0] & 1 == 0
    }

    /// Checks `self` is greater or equal then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn ct_ge(&self, rhs: &Self) -> bool {
        ct_for_unroll6!((i in 0..N) {
            let a = self.limbs[N - i - 1];
            let b = rhs.limbs[N - i - 1];
            if a > b {
                return true;
            } else if a < b {
                return false;
            }
        });
        true
    }

    /// Checks `self` is greater then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn ct_gt(&self, rhs: &Self) -> bool {
        ct_for_unroll6!((i in 0..N) {
            let a = self.limbs[N - i - 1];
            let b = rhs.limbs[N - i - 1];
            if a > b {
                return true;
            } else if a < b {
                return false;
            }
        });
        false
    }

    /// Checks `self` is less or equal then `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn ct_le(&self, rhs: &Self) -> bool {
        ct_for_unroll6!((i in 0..N) {
            let a = self.limbs[N - i - 1];
            let b = rhs.limbs[N - i - 1];
            if a < b {
                return true;
            } else if a > b {
                return false;
            }
        });
        true
    }

    /// Checks `self` is less then `rhs` (constant).
    #[must_use]
    pub const fn ct_lt(&self, rhs: &Self) -> bool {
        ct_for_unroll6!((i in 0..N) {
            let a = self.limbs[N - i - 1];
            let b = rhs.limbs[N - i - 1];
            if a < b {
                return true;
            } else if a > b {
                return false;
            }
        });
        false
    }

    /// Checks `self` is zero (constant).
    #[must_use]
    pub const fn ct_is_zero(&self) -> bool {
        self.ct_eq(&Self::ZERO)
    }

    /// Checks if `self` is equal to `rhs` (constant).
    #[must_use]
    pub const fn ct_eq(&self, rhs: &Self) -> bool {
        ct_for!((i in 0..N) {
            if self.limbs[i] != rhs.limbs[i] {
                return false;
            }
        });
        true
    }

    /// Checks if `self` is not equal to `rhs` (constant).
    #[must_use]
    #[inline(always)]
    pub const fn ct_ne(&self, rhs: &Self) -> bool {
        !self.ct_eq(rhs)
    }

    /// Return the minimum number of bits needed to encode this number.
    #[doc(hidden)]
    #[must_use]
    pub const fn ct_num_bits(&self) -> usize {
        // Total number of bits.
        let mut num_bits = Self::BITS;

        // Start with the last (highest) limb.
        let mut index = N - 1;
        loop {
            // Subtract leading zeroes, from the total number of limbs.
            let leading = self.limbs[index].leading_zeros() as usize;
            num_bits -= leading;

            // If the limb is not empty, stop processing other limbs.
            if leading != 64 {
                break;
            }

            if index == 0 {
                break;
            }
            index -= 1;
        }

        // And return the result.
        num_bits
    }

    /// Find the `i`-th bit of `self`.
    #[must_use]
    pub const fn ct_get_bit(&self, i: usize) -> bool {
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
        ct_for_unroll6!((i in 0..N) {
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
    const fn ct_checked_mul2(mut self) -> (Self, bool) {
        let mut last = 0;
        ct_for!((i in 0..N) {
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
    pub const fn ct_checked_sub(mut self, rhs: &Self) -> (Self, bool) {
        let mut borrow = 0;

        ct_for_unroll6!((i in 0..N) {
            (self.limbs[i], borrow) = limb::sbb(self.limbs[i], rhs.limbs[i], borrow);
        });

        (self, borrow != 0)
    }

    /// Subtract `rhs` from `self`, returning the result wrapping around the
    /// lower boundary (constant).
    #[inline(always)]
    #[must_use]
    pub const fn ct_wrapping_sub(&self, rhs: &Self) -> Self {
        self.ct_checked_sub(rhs).0
    }

    /// Add `rhs` to `self`, returning the result and whether overflow occurred
    /// (constant).
    #[inline]
    #[must_use]
    pub const fn ct_checked_add(mut self, rhs: &Self) -> (Self, bool) {
        let mut carry = 0;

        ct_for!((i in 0..N) {
            (self.limbs[i], carry) = limb::adc(self.limbs[i], rhs.limbs[i], carry);
        });

        (self, carry != 0)
    }

    /// Add `rhs` to `self` in-place, returning whether overflow occurred.
    #[inline(always)]
    pub fn checked_add_assign(&mut self, rhs: &Self) -> bool {
        let mut carry = false;

        ct_for_unroll6!((i in 0..N) {
            carry = limb::adc_assign(&mut self.limbs[i], rhs.limbs[i], carry);
        });

        carry
    }

    /// Subtract `rhs` from `self` in-place, returning whether overflow
    /// occurred.
    #[inline(always)]
    pub fn checked_sub_assign(&mut self, rhs: &Self) -> bool {
        let mut borrow = false;

        ct_for_unroll6!((i in 0..N) {
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
    /// like [`ct_for_unroll6`].
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Multiplication_algorithm
    #[inline(always)]
    #[must_use]
    pub const fn ct_widening_mul(&self, rhs: &Self) -> (Self, Self) {
        let (mut lo, mut hi) = ([0u64; N], [0u64; N]);
        // For each digit of the first number,
        ct_for_unroll6!((i in 0..N) {
            let mut carry = 0;
            // perform multiplication of each digit from the second.
            ct_for_unroll6!((j in 0..N) {
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
    #[must_use]
    pub const fn ct_mul(&self, rhs: &Self) -> Self {
        let (low, high) = self.ct_widening_mul(rhs);
        assert!(high.ct_eq(&Uint::<N>::ZERO), "overflow on multiplication");
        low
    }

    /// Add two numbers and panic on overflow.
    #[must_use]
    pub const fn ct_add(&self, rhs: &Self) -> Self {
        let (low, carry) = self.ct_adc(rhs, Limb::ZERO);
        assert!(carry == 0, "overflow on addition");
        low
    }

    /// Add two numbers wrapping around the upper boundary.
    #[must_use]
    pub const fn ct_wrapping_add(&self, rhs: &Self) -> Self {
        let (low, _) = self.ct_adc(rhs, Limb::ZERO);
        low
    }

    /// Computes `a + b + carry`, returning the result along with the new carry.
    #[inline(always)]
    #[must_use]
    pub const fn ct_adc(&self, rhs: &Uint<N>, mut carry: Limb) -> (Self, Limb) {
        let mut limbs = [Limb::ZERO; N];

        ct_for!((i in 0..N) {
            (limbs[i], carry) = limb::adc(self.limbs[i], rhs.limbs[i], carry);
        });

        (Self { limbs }, carry)
    }

    /// Create a new [`Uint`] from the provided little endian bytes.
    #[must_use]
    pub const fn ct_from_le_slice(bytes: &[u8]) -> Self {
        const LIMB_BYTES: usize = Limb::BITS as usize / 8;
        assert!(
            bytes.len() == LIMB_BYTES * N,
            "bytes are not the expected size"
        );

        let mut res = [Limb::ZERO; N];
        let mut buf = [0u8; LIMB_BYTES];

        ct_for!((i in 0..N) {
            ct_for!((j in 0..LIMB_BYTES) {
                buf[j] = bytes[i * LIMB_BYTES + j];
            });
            res[i] = Limb::from_le_bytes(buf);
        });

        Self::new(res)
    }
}

// ----------- From Impls -----------

/// Constant implementation from primitives.
macro_rules! impl_ct_from_primitive {
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
impl_ct_from_primitive!(u8, from_u8);
impl_ct_from_primitive!(u16, from_u16);
impl_ct_from_primitive!(u32, from_u32);
impl_ct_from_primitive!(u64, from_u64);
impl_ct_from_primitive!(usize, from_usize);

// Logic for `u128` conversion is different from `u8`..`u64`, due to the size of
// the `Limb`.
impl<const N: usize> Uint<N> {
    /// Create a [`Uint`] from a `u128` integer (constant).
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_lossless)]
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
        ct_for_unroll6!((i in 0..N) {
            let a = &self.limbs[N - i - 1];
            let b = &rhs.limbs[N - i - 1];
            match a.cmp(b) {
                Ordering::Equal => {}
                order => return order,
            };
        });

        Ordering::Equal
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
        self.ct_is_odd()
    }

    fn is_even(&self) -> bool {
        self.ct_is_even()
    }

    fn is_zero(&self) -> bool {
        self.ct_is_zero()
    }

    fn num_bits(&self) -> usize {
        self.ct_num_bits()
    }

    fn get_bit(&self, i: usize) -> bool {
        self.ct_get_bit(i)
    }

    fn from_bytes_le(bytes: &[u8]) -> Self {
        Self::ct_from_le_slice(bytes)
    }

    fn into_bytes_le(self) -> Vec<u8> {
        self.limbs.iter().flat_map(|&limb| limb.to_le_bytes()).collect()
    }
}

impl<const N: usize> BitIteratorBE for Uint<N> {
    fn bit_be_iter(&self) -> impl Iterator<Item = bool> {
        self.as_limbs().iter().rev().flat_map(Limb::bit_be_iter)
    }
}

/// Parse a number from a string in a given radix.
///
/// This implementation can be slow on big numbers and possibly fail constant
/// compilation by timeout.
///
/// I.e., convert string encoded integer `s` to base-`radix` number.
#[must_use]
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
        uint = uint.ct_add(&digit.ct_mul(&order));

        // If we reached the beginning of the string, return the number.
        if index == 0 {
            return uint;
        }

        // Increase the order of magnitude.
        order = uint_radix.ct_mul(&order);

        // Move to the next digit.
        index -= 1;
    }
}

/// Parse a number from a hex string.
///
/// This implementation performs faster than [`from_str_radix`], since it
/// assumes the radix is already `16`.
///
/// If the string number is shorter, then [`Uint`] can store.
/// Returns a [`Uint`] with leading zeroes.
#[must_use]
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

        // Since a base-16 digit can be represented with the same bits, we can
        // copy these bits.
        let digit_mask = digit << ((num_index % digits_in_limb) * digit_size);
        num[(num_index / digits_in_limb) as usize] |= digit_mask;

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

/// Parse a single UTF-8 byte.
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

    /// Compute a reminder of division `self` by `rhs` (constant).
    ///
    /// Basic division algorithm based on [wiki].
    /// Fine to be used for constant evaluation, but slow in runtime.
    ///
    /// [wiki]: https://en.wikipedia.org/wiki/Division_algorithm
    #[must_use]
    pub const fn ct_rem(&self, rhs: &Uint<N>) -> Uint<N> {
        assert!(!rhs.ct_is_zero(), "should not divide by zero");

        let mut remainder = Uint::<N>::ZERO;

        // Start with the last bit.
        let mut index = self.ct_num_bits() - 1;
        loop {
            // Shift the remainder to the left by 1,
            let (result, carry) = remainder.ct_checked_mul2();
            remainder = result;

            // and set the first bit to reminder from the dividend.
            remainder.limbs[0] |= self.ct_get_bit(index) as Limb;

            // If the remainder overflows, subtract the divisor.
            if remainder.ct_ge(rhs) || carry {
                (remainder, _) = remainder.ct_checked_sub(rhs);
            }

            if index == 0 {
                break remainder;
            }
            index -= 1;
        }
    }

    /// Find the number of bits in the binary decomposition of `self`.
    #[must_use]
    pub const fn ct_num_bits(&self) -> usize {
        let high_num_bits = self.high.ct_num_bits();
        if high_num_bits == 0 {
            self.low.ct_num_bits()
        } else {
            high_num_bits + Uint::<N>::BITS
        }
    }

    /// Compute the `i`-th bit of `self`.
    #[must_use]
    pub const fn ct_get_bit(&self, i: usize) -> bool {
        if i >= Uint::<N>::BITS {
            self.high.ct_get_bit(i - Uint::<N>::BITS)
        } else {
            self.low.ct_get_bit(i)
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use proptest::prelude::*;

    use crate::{
        arithmetic::{
            uint::{from_str_hex, from_str_radix, Uint, WideUint},
            *,
        },
        bits::BitIteratorBE,
    };

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
    fn ct_rem() {
        let dividend = from_num!("43129923721897334698312931");
        let divisor = from_num!("375923422");
        let result =
            WideUint::<4>::new(dividend, Uint::<4>::ZERO).ct_rem(&divisor);
        assert_eq!(result, from_num!("216456157"));
    }

    #[test]
    fn ct_ge_le_gt_lt_eq_ne() {
        let a: Uint<4> = Uint::new([0, 0, 0, 5]);
        let b: Uint<4> = Uint::new([4, 0, 0, 0]);
        assert!(a.ct_ge(&b));
        assert!(a.ct_gt(&b));
        assert!(!a.ct_le(&b));
        assert!(!a.ct_lt(&b));
        assert!(!a.ct_eq(&b));
        assert!(a.ct_ne(&b));

        let a: Uint<4> = Uint::new([0, 0, 0, 5]);
        let b: Uint<4> = Uint::new([0, 0, 0, 6]);
        assert!(!a.ct_ge(&b));
        assert!(!a.ct_gt(&b));
        assert!(a.ct_le(&b));
        assert!(a.ct_lt(&b));
        assert!(!a.ct_eq(&b));
        assert!(a.ct_ne(&b));

        let a: Uint<4> = Uint::new([0, 0, 1, 2]);
        let b: Uint<4> = Uint::new([0, 0, 1, 2]);
        assert!(a.ct_ge(&b));
        assert!(!a.ct_gt(&b));
        assert!(a.ct_le(&b));
        assert!(!a.ct_lt(&b));
        assert!(a.ct_eq(&b));
        assert!(!a.ct_ne(&b));
    }
}
