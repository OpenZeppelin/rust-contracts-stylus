//  TODO#q: Odd<Uint<N>> - Odd numbers.
//  TODO#q: NonZero<Uint<N>> - Non zero numbers for division.

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
use zeroize::Zeroize;

use crate::{
    arithmetic::{
        limb,
        limb::{
            adc, adc_for_add_with_carry, sbb, sbb_for_sub_with_borrow, Limb,
            Limbs,
        },
        BigInteger,
    },
    bits::BitIteratorBE,
    const_for, unroll6_for,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Zeroize)]
pub struct Uint<const N: usize> {
    pub(crate) limbs: Limbs<N>,
}

impl<const N: usize> Default for Uint<N> {
    fn default() -> Self {
        Self { limbs: [0u64; N] }
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
    pub const fn new(value: [u64; N]) -> Self {
        Self { limbs: value }
    }

    pub const fn as_limbs(&self) -> &[Limb; N] {
        &self.limbs
    }

    // TODO#q: add another conversions from u8, u16 and so on
    pub const fn from_u32(val: u32) -> Self {
        let mut repr = Self::ZERO;
        repr.limbs[0] = val as u64;
        repr
    }

    #[doc(hidden)]
    pub const fn ct_is_even(&self) -> bool {
        self.limbs[0] % 2 == 0
    }

    #[doc(hidden)]
    pub const fn ct_is_odd(&self) -> bool {
        self.limbs[0] % 2 == 1
    }

    #[doc(hidden)]
    pub const fn mod_4(&self) -> u8 {
        // To compute n % 4, we need to simply look at the
        // 2 least significant bits of n, and check their value mod 4.
        (((self.limbs[0] << 62) >> 62) % 4) as u8
    }

    /// Compute a right shift of `self`
    /// This is equivalent to a (saturating) division by 2.
    #[doc(hidden)]
    pub const fn ct_shr(&self) -> Self {
        let mut result = *self;
        let mut t = 0;
        const_for!((i in 0..N) {
            let a = result.limbs[N - i - 1];
            let t2 = a << 63;
            result.limbs[N - i - 1] >>= 1;
            result.limbs[N - i - 1] |= t;
            t = t2;
        });
        result
    }

    const fn ct_geq(&self, other: &Self) -> bool {
        const_for!((i in 0..N) {
            let a = self.limbs[N - i - 1];
            let b = other.limbs[N - i - 1];
            if a < b {
                return false;
            } else if a > b {
                return true;
            }
        });
        true
    }

    /// Return the minimum number of bits needed to encode this number.
    #[doc(hidden)]
    pub const fn ct_num_bits(self) -> usize {
        // Total number of bits.
        let mut num_bits = Self::BITS;

        // Start with the last (highest) limb.
        let mut index = N - 1;
        loop {
            // Subtract leading zeroes, from the total number of limbs.
            let leading = self.limbs[index].leading_zeros() as usize;
            num_bits -= leading;

            // If the limb is not empty, stop processing other limbs,
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

    #[inline]
    pub(crate) const fn ct_sub_with_borrow(
        mut self,
        other: &Self,
    ) -> (Self, bool) {
        let mut borrow = 0;

        const_for!((i in 0..N) {
            (self.limbs[i], borrow) = sbb(self.limbs[i], other.limbs[i], borrow);
        });

        (self, borrow != 0)
    }

    #[inline]
    #[allow(unused)]
    pub(crate) fn mul2(&mut self) -> bool {
        let mut last = 0;
        for i in 0..N {
            let a = &mut self.limbs[i];
            let tmp = *a >> 63;
            *a <<= 1;
            *a |= last;
            last = tmp;
        }
        last != 0
    }

    #[inline]
    pub(crate) const fn ct_add_with_carry(
        mut self,
        other: &Self,
    ) -> (Self, bool) {
        let mut carry = 0;

        const_for!((i in 0..N) {
            (self.limbs[i], carry) = adc(self.limbs[i], other.limbs[i], carry);
        });

        (self, carry != 0)
    }

    const fn ct_mul2_with_carry(mut self) -> (Self, bool) {
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

    pub(crate) const fn ct_is_zero(&self) -> bool {
        let mut is_zero = true;
        const_for!((i in 0..N) {
            is_zero &= self.limbs[i] == 0;
        });
        is_zero
    }

    pub fn div2(&mut self) {
        let mut t = 0;
        for a in self.limbs.iter_mut().rev() {
            let t2 = *a << 63;
            *a >>= 1;
            *a |= t;
            t = t2;
        }
    }

    // TODO#q: rename to checked_add?
    #[inline(always)]
    pub(crate) fn add_with_carry(&mut self, other: &Self) -> bool {
        let mut carry = false;

        unroll6_for!((i in 0..N) {
            carry = adc_for_add_with_carry(&mut self.limbs[i], other.limbs[i], carry);
        });

        carry
    }

    #[inline(always)]
    pub(crate) fn sub_with_borrow(&mut self, other: &Self) -> bool {
        let mut borrow = false;

        unroll6_for!((i in 0..N) {
            borrow =
                sbb_for_sub_with_borrow(&mut self.limbs[i], other.limbs[i], borrow);
        });

        borrow
    }

    /// Compute "wide" multiplication, with a product twice the size of the
    /// input.
    ///
    /// Returns a tuple containing the `(lo, hi)` components of the product.
    #[inline(always)]
    pub const fn ct_mul_wide(&self, rhs: &Self) -> (Self, Self) {
        // TODO#q: document implementation
        let (mut lo, mut hi) = ([0u64; N], [0u64; N]);
        unroll6_for!((i in 0..N) {
            let mut carry = 0;
            unroll6_for!((j in 0..N) {
                let k = i + j;
                if k >= N {
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
            hi[i] = carry;
        });

        (Self::new(lo), Self::new(hi))
    }

    /// Multiply two numbers and panic on overflow.
    #[must_use]
    pub const fn ct_mul(&self, rhs: &Self) -> Self {
        let (low, high) = self.ct_mul_wide(rhs);
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

    /// Add two numbers wrapping around upper boundery.
    #[must_use]
    pub const fn ct_wrapping_add(&self, rhs: &Self) -> Self {
        let (low, _) = self.ct_adc(rhs, Limb::ZERO);
        low
    }

    pub const fn ct_ge(&self, rhs: &Self) -> bool {
        const_for!((i in 0..N) {
            if self.limbs[i] < rhs.limbs[i] {
                return false;
            } else if self.limbs[i] > rhs.limbs[i] {
                return true;
            }
        });
        true
    }

    // TODO#q: unify ct_eq with ct_is_zero
    pub const fn ct_eq(&self, rhs: &Self) -> bool {
        const_for!((i in 0..N) {
            if self.limbs[i] != rhs.limbs[i] {
                return false;
            }
        });
        true
    }

    #[inline(always)]
    /// Computes `a + b + carry`, returning the result along with the new carry.
    // NOTE#q: crypto_bigint
    pub const fn ct_adc(&self, rhs: &Uint<N>, mut carry: Limb) -> (Self, Limb) {
        let mut limbs = [Limb::ZERO; N];
        let mut i = 0;

        while i < N {
            let (w, c) = limb::ct_adc(self.limbs[i], rhs.limbs[i], carry);
            limbs[i] = w;
            carry = c;
            i += 1;
        }

        (Self { limbs }, carry)
    }

    /// Create a new [`Uint`] from the provided little endian bytes.
    // NOTE#q: crypto_bigint
    pub const fn ct_from_le_slice(bytes: &[u8]) -> Self {
        const LIMB_BYTES: usize = Limb::BITS as usize / 8;
        assert!(
            bytes.len() == LIMB_BYTES * N,
            "bytes are not the expected size"
        );

        let mut res = [Limb::ZERO; N];
        let mut buf = [0u8; LIMB_BYTES];
        let mut i = 0;

        while i < N {
            let mut j = 0;
            while j < LIMB_BYTES {
                buf[j] = bytes[i * LIMB_BYTES + j];
                j += 1;
            }
            res[i] = Limb::from_le_bytes(buf);
            i += 1;
        }

        Self::new(res)
    }
}

// ----------- Traits Impls -----------

impl<const N: usize> UpperHex for Uint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016X}", BigUint::from(*self))
    }
}

impl<const N: usize> Debug for Uint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", BigUint::from(*self))
    }
}

impl<const N: usize> Display for Uint<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", BigUint::from(*self))
    }
}

impl<const N: usize> Ord for Uint<N> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        unroll6_for!((i in 0..N) {
            let a = &self.limbs[N - i - 1];
            let b = &other.limbs[N - i - 1];
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
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
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

impl<const N: usize> From<u64> for Uint<N> {
    #[inline]
    fn from(val: u64) -> Uint<N> {
        let mut repr = Self::default();
        repr.limbs[0] = val;
        repr
    }
}

impl<const N: usize> From<u32> for Uint<N> {
    #[inline]
    fn from(val: u32) -> Uint<N> {
        let mut repr = Self::default();
        repr.limbs[0] = val.into();
        repr
    }
}

impl<const N: usize> From<u16> for Uint<N> {
    #[inline]
    fn from(val: u16) -> Uint<N> {
        let mut repr = Self::default();
        repr.limbs[0] = val.into();
        repr
    }
}

impl<const N: usize> From<u8> for Uint<N> {
    #[inline]
    fn from(val: u8) -> Uint<N> {
        let mut repr = Self::default();
        repr.limbs[0] = val.into();
        repr
    }
}

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

// TODO#q: Implement rand Distribution
/*impl<const N: usize> Distribution<BigInt<N>> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BigInt<N> {
        BigInt([(); N].map(|_| rng.gen()))
    }
}*/

// TODO#q: remove num_bigint::BigUint conversion
impl<const N: usize> From<Uint<N>> for BigUint {
    #[inline]
    fn from(val: Uint<N>) -> num_bigint::BigUint {
        BigUint::from_bytes_le(&val.into_bytes_le())
    }
}

impl<const N: usize> From<Uint<N>> for num_bigint::BigInt {
    #[inline]
    fn from(val: Uint<N>) -> num_bigint::BigInt {
        use num_bigint::Sign;
        let sign = if val.is_zero() { Sign::NoSign } else { Sign::Plus };
        num_bigint::BigInt::from_bytes_le(sign, &val.into_bytes_le())
    }
}

impl<B: Borrow<Self>, const N: usize> BitXorAssign<B> for Uint<N> {
    fn bitxor_assign(&mut self, rhs: B) {
        (0..N).for_each(|i| self.limbs[i] ^= rhs.borrow().limbs[i])
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
        (0..N).for_each(|i| self.limbs[i] &= rhs.borrow().limbs[i])
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
        (0..N).for_each(|i| self.limbs[i] |= rhs.borrow().limbs[i])
    }
}

impl<B: Borrow<Self>, const N: usize> BitOr<B> for Uint<N> {
    type Output = Self;

    fn bitor(mut self, rhs: B) -> Self::Output {
        self |= rhs;
        self
    }
}

// TODO#q: Document ShrAssign and ShlAssign implementations

impl<const N: usize> ShrAssign<u32> for Uint<N> {
    /// Computes the bitwise shift right operation in place.
    ///
    /// Differently from the built-in numeric types (u8, u32, u64, etc.) this
    /// operation does *not* return an underflow error if the number of bits
    /// shifted is larger than N * 64. Instead the result will be saturated to
    /// zero.
    fn shr_assign(&mut self, mut rhs: u32) {
        if rhs >= (64 * N) as u32 {
            *self = Self::from(0u64);
            return;
        }

        while rhs >= 64 {
            let mut t = 0;
            for limb in self.limbs.iter_mut().rev() {
                core::mem::swap(&mut t, limb);
            }
            rhs -= 64;
        }

        if rhs > 0 {
            let mut t = 0;
            for a in self.limbs.iter_mut().rev() {
                let t2 = *a << (64 - rhs);
                *a >>= rhs;
                *a |= t;
                t = t2;
            }
        }
    }
}

impl<const N: usize> Shr<u32> for Uint<N> {
    type Output = Self;

    /// Computes bitwise shift right operation.
    ///
    /// Differently from the built-in numeric types (u8, u32, u64, etc.) this
    /// operation does *not* return an underflow error if the number of bits
    /// shifted is larger than N * 64. Instead the result will be saturated to
    /// zero.
    fn shr(mut self, rhs: u32) -> Self::Output {
        self >>= rhs;
        self
    }
}

impl<const N: usize> ShlAssign<u32> for Uint<N> {
    /// Computes the bitwise shift left operation in place.
    ///
    /// Differently from the built-in numeric types (u8, u32, u64, etc.) this
    /// operation does *not* return an overflow error if the number of bits
    /// shifted is larger than N * 64. Instead, the overflow will be chopped
    /// off.
    fn shl_assign(&mut self, mut rhs: u32) {
        if rhs >= (64 * N) as u32 {
            *self = Self::from(0u64);
            return;
        }

        while rhs >= 64 {
            let mut t = 0;
            for i in 0..N {
                core::mem::swap(&mut t, &mut self.limbs[i]);
            }
            rhs -= 64;
        }

        if rhs > 0 {
            let mut t = 0;
            #[allow(unused)]
            for i in 0..N {
                let a = &mut self.limbs[i];
                let t2 = *a >> (64 - rhs);
                *a <<= rhs;
                *a |= t;
                t = t2;
            }
        }
    }
}

impl<const N: usize> Shl<u32> for Uint<N> {
    type Output = Self;

    /// Computes the bitwise shift left operation in place.
    ///
    /// Differently from the built-in numeric types (u8, u32, u64, etc.) this
    /// operation does *not* return an overflow error if the number of bits
    /// shifted is larger than N * 64. Instead, the overflow will be chopped
    /// off.
    fn shl(mut self, rhs: u32) -> Self::Output {
        self <<= rhs;
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
    const BITS: usize = Self::NUM_LIMBS * (Limb::BITS as usize);
    const MAX: Self = Self { limbs: [u64::MAX; N] };
    const NUM_LIMBS: usize = N;
    const ONE: Self = {
        let mut one = Self::ZERO;
        one.limbs[0] = 1;
        one
    };
    const ZERO: Self = Self { limbs: [0u64; N] };

    fn is_odd(&self) -> bool {
        self.limbs[0] & 1 == 1
    }

    fn is_even(&self) -> bool {
        !self.is_odd()
    }

    fn is_zero(&self) -> bool {
        self.limbs.iter().all(Zero::is_zero)
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

    fn into_bytes_le(self) -> alloc::vec::Vec<u8> {
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
    pub const fn new(low: Uint<N>, high: Uint<N>) -> Self {
        Self { low, high }
    }

    /// Compute reminder of division of `self` by `rhs`.
    pub const fn ct_rem(&self, rhs: &Uint<N>) -> Uint<N> {
        assert!(!rhs.ct_is_zero(), "should not divide by zero");

        let mut remainder = Uint::<N>::ZERO;
        let mut index = self.ct_num_bits() - 1;
        let mut carry = false;
        loop {
            (remainder, carry) = remainder.ct_mul2_with_carry();
            remainder.limbs[0] |= self.ct_get_bit(index) as Limb;
            if remainder.ct_geq(rhs) || carry {
                let (r, borrow) = remainder.ct_sub_with_borrow(rhs);
                remainder = r;
                // TODO#q: add doc comment and describe ct_rem
                assert!(borrow == carry);
            }

            if index == 0 {
                break remainder;
            }
            index -= 1;
        }
    }

    /// Find the number of bits in the binary decomposition of `self`.
    #[doc(hidden)]
    pub const fn ct_num_bits(&self) -> usize {
        let high_num_bits = self.high.ct_num_bits();
        if high_num_bits == 0 {
            self.low.ct_num_bits()
        } else {
            high_num_bits + Uint::<N>::BITS
        }
    }

    /// Compute the `i`-th bit of `self`.
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
    use proptest::proptest;

    use crate::arithmetic::{
        uint::{from_str_hex, from_str_radix, Uint},
        *,
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
        proptest!(|(s in "[0-9a-fA-F]{1,64}")| {
            let uint_from_hex: Uint<4> = from_str_hex(&s);
            let expected: Uint<4> = from_str_radix(&s, 16);
            assert_eq!(uint_from_hex, expected);
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
}
