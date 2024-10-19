use alloc::vec;

use ark_std::{
    borrow::Borrow,
    fmt::{Debug, Display, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl,
        ShlAssign, Shr, ShrAssign,
    },
    str::FromStr,
    vec::*,
};
use num_bigint::BigUint;
use zeroize::Zeroize;

use crate::bits::{BitIteratorBE, BitIteratorLE};

/// Compute the signed modulo operation on a u64 representation, returning the
/// result. If n % modulus > modulus / 2, return modulus - n
pub fn signed_mod_reduction(n: u64, modulus: u64) -> i64 {
    let t = (n % modulus) as i64;
    if t as u64 >= (modulus / 2) {
        t - (modulus as i64)
    } else {
        t
    }
}

/// This defines a `BigInteger`, a smart wrapper around a
/// sequence of `u64` limbs, least-significant limb first.
// TODO#q: get rid of this trait once we can use associated constants in const
// generics.
pub trait BigInteger:
    // CanonicalSerialize
    // + CanonicalDeserialize
    Copy
    + Clone
    + Debug
    + Default
    + Display
    + Eq
    + Ord
    + Send
    + Sized
    + Sync
    + 'static
    // + UniformRand
    + Zeroize
    + AsMut<[u64]>
    + AsRef<[u64]>
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
    + TryFrom<BigUint, Error = ()>
    + FromStr
    + Into<BigUint>
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
    + Shr<u32, Output = Self>
    + ShrAssign<u32>
    + Shl<u32, Output = Self>
    + ShlAssign<u32>
{
    /// Number of 64-bit limbs representing `Self`.
    const NUM_LIMBS: usize;

    /// Add another [`BigInteger`] to `self`. This method stores the result in `self`,
    /// and returns a carry bit.
    fn add_with_carry(&mut self, other: &Self) -> bool;

    /// Subtract another [`BigInteger`] from this one. This method stores the result in
    /// `self`, and returns a borrow.
    fn sub_with_borrow(&mut self, other: &Self) -> bool;

    /// Performs a leftwise bitshift of this number, effectively multiplying
    /// it by 2. Overflow is ignored.
    fn mul2(&mut self) -> bool;

    /// Performs a leftwise bitshift of this number by n bits, effectively multiplying
    /// it by 2^n. Overflow is ignored.
    /// # Example
    #[deprecated(since = "0.4.2", note = "please use the operator `<<` instead")]
    fn muln(&mut self, amt: u32);

    /// Multiplies this [`BigInteger`] by another `BigInteger`, storing the result in `self`.
    /// Overflow is ignored.
    fn mul_low(&self, other: &Self) -> Self;

    /// Multiplies this [`BigInteger`] by another `BigInteger`, returning the high bits of the result.
    fn mul_high(&self, other: &Self) -> Self;

    /// Multiplies this [`BigInteger`] by another `BigInteger`, returning both low and high bits of the result.
    fn mul(&self, other: &Self) -> (Self, Self);

    /// Performs a rightwise bitshift of this number, effectively dividing
    /// it by 2.
    fn div2(&mut self);

    /// Performs a rightwise bitshift of this number by some amount.
    /// # Example
    #[deprecated(since = "0.4.2", note = "please use the operator `>>` instead")]
    fn divn(&mut self, amt: u32);

    /// Returns true iff this number is odd.
    fn is_odd(&self) -> bool;

    /// Returns true iff this number is even.
    fn is_even(&self) -> bool;

    /// Returns true iff this number is zero.
    fn is_zero(&self) -> bool;

    /// Compute the minimum number of bits needed to encode this number.
    fn num_bits(&self) -> u32;

    /// Compute the `i`-th bit of `self`.
    fn get_bit(&self, i: usize) -> bool;

    /// Returns the big integer representation of a given big endian boolean
    /// array.
    fn from_bits_be(bits: &[bool]) -> Self;

    /// Returns the big integer representation of a given little endian boolean
    /// array.
    fn from_bits_le(bits: &[bool]) -> Self;

    /// Returns the bit representation in a big endian boolean array,
    /// with leading zeroes.
    fn to_bits_be(&self) -> Vec<bool> {
        BitIteratorBE::new(self).collect::<Vec<_>>()
    }

    /// Returns the bit representation in a little endian boolean array,
    /// with trailing zeroes.
    fn to_bits_le(&self) -> Vec<bool> {
        BitIteratorLE::new(self).collect::<Vec<_>>()
    }

    /// Returns the byte representation in a big endian byte array,
    /// with leading zeros.
    fn to_bytes_be(&self) -> Vec<u8>;

    /// Returns the byte representation in a little endian byte array,
    /// with trailing zeros.
    fn to_bytes_le(&self) -> Vec<u8>;

    /// Returns the windowed non-adjacent form of `self`, for a window of size `w`.
    fn find_wnaf(&self, w: usize) -> Option<Vec<i64>> {
        // w > 2 due to definition of wNAF, and w < 64 to make sure that `i64`
        // can fit each signed digit
        if (2..64).contains(&w) {
            let mut res = vec![];
            let mut e = *self;

            while !e.is_zero() {
                let z: i64;
                if e.is_odd() {
                    z = signed_mod_reduction(e.as_ref()[0], 1 << w);
                    if z >= 0 {
                        e.sub_with_borrow(&Self::from(z as u64));
                    } else {
                        e.add_with_carry(&Self::from((-z) as u64));
                    }
                } else {
                    z = 0;
                }
                res.push(z);
                e.div2();
            }

            Some(res)
        } else {
            None
        }
    }
}
