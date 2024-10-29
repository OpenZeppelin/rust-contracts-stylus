use core::{
    fmt::{Debug, Display, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl,
        ShlAssign, Shr, ShrAssign,
    },
};

#[allow(clippy::module_name_repetitions)]
pub use crypto_bigint;
use crypto_bigint::{Integer, Limb, Uint, Zero};
use zeroize::Zeroize;

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

    /// Returns true iff this number is odd.
    /// # Example
    ///
    /// ```
    /// use openzeppelin_crypto::bigint::{BigInteger, crypto_bigint::U64};
    ///
    /// let mut one = U64::from(1u64);
    /// assert!(one.is_odd());
    /// ```
    fn is_odd(&self) -> bool;

    /// Returns true iff this number is even.
    /// # Example
    ///
    /// ```
    /// use openzeppelin_crypto::bigint::{BigInteger, crypto_bigint::U64};
    ///
    /// let mut two = U64::from(2u64);
    /// assert!(two.is_even());
    /// ```
    fn is_even(&self) -> bool;

    /// Returns true iff this number is zero.
    /// # Example
    ///
    /// ```
    /// use openzeppelin_crypto::bigint::{BigInteger, crypto_bigint::U64};
    ///
    /// let mut zero = U64::from(0u64);
    /// assert!(zero.is_zero());
    /// ```
    fn is_zero(&self) -> bool;

    /// Compute the minimum number of bits needed to encode this number.
    /// # Example
    /// ```
    /// use openzeppelin_crypto::bigint::{BigInteger, crypto_bigint::U64};
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
    /// use openzeppelin_crypto::bigint::{BigInteger, crypto_bigint::U64};
    ///
    /// let mut one = U64::from(1u64);
    /// assert!(one.get_bit(0));
    /// assert!(!one.get_bit(1));
    /// ```
    fn get_bit(&self, i: usize) -> bool;
}

impl<const N: usize> BigInteger for Uint<N> {
    const NUM_LIMBS: usize = N;

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

/// Parse a number from a string in a given radix.
///
/// I.e., convert string encoded integer `s` to base-`radix` number.
#[must_use]
pub const fn from_str_radix<const LIMBS: usize>(
    s: &str,
    radix: u32,
) -> Uint<LIMBS> {
    let bytes = s.as_bytes();

    // The lowest order number is at the end of the string.
    // Begin parsing from the last index of the string.
    let mut index = bytes.len() - 1;

    let mut uint = Uint::from_u32(0);
    let mut order = Uint::from_u32(1);
    let uint_radix = Uint::from_u32(radix);

    loop {
        let ch = parse_utf8_byte(bytes[index]);
        let digit = match ch.to_digit(radix) {
            None => {
                panic!("invalid digit");
            }
            Some(digit) => Uint::from_u32(digit),
        };

        // Add a digit multiplied by order.
        uint = add(&uint, &mul(&digit, &order));

        // Increase the order of magnitude.
        order = mul(&uint_radix, &order);

        if index == 0 {
            return uint;
        }
        index -= 1;
    }
}

/// Multiply two numbers and panic on overflow.
#[must_use]
pub const fn mul<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    b: &Uint<LIMBS>,
) -> Uint<LIMBS> {
    let (low, high) = a.mul_wide(&b);
    assert!(high.bits() == 0, "overflow on multiplication");
    low
}

/// Add two numbers and panic on overflow.
#[must_use]
pub const fn add<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    b: &Uint<LIMBS>,
) -> Uint<LIMBS> {
    let (low, carry) = a.adc(b, Limb::ZERO);
    assert!(carry.0 == 0, "overflow on addition");
    low
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
        $crate::bigint::from_str_radix($num, 10)
    };
}

/// This macro converts a string hex number to a big integer.
#[macro_export]
macro_rules! from_hex {
    ($num:literal) => {
        $crate::bigint::crypto_bigint::Uint::from_be_hex($num)
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str_radix() {
        let uint = from_str_radix::<4>("28948022309329048855892746252171976963363056481941647379679742748393362948097", 10);
        #[allow(clippy::unreadable_literal)]
        let expected = Uint::<4>::from_words([
            10108024940646105089u64,
            2469829653919213789u64,
            0u64,
            4611686018427387904u64,
        ]);
        assert_eq!(uint, expected);
    }
}
