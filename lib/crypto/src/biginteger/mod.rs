use core::{
    fmt::{Debug, Display, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl,
        ShlAssign, Shr, ShrAssign,
    },
};

use zeroize::Zeroize;

/// This defines a `BigInteger`, a smart wrapper around a
/// sequence of `u64` limbs, least-significant limb first.
// TODO: get rid of this trait once we can use associated constants in const
// generics.
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
    // + AsMut<[u64]> // TODO#q: think how to hold a reference to bytes
    // + AsRef<[u64]>
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
    /// Number of 64-bit limbs representing `Self`.
    const NUM_LIMBS: usize;

    /// Add another [`BigInteger`] to `self`. This method stores the result in
    /// `self`, and returns a carry bit.
    ///
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let (mut one, mut x) = (B::from(1u64), B::from(2u64));
    /// let carry = x.add_with_carry(&one);
    /// assert_eq!(x, B::from(3u64));
    /// assert_eq!(carry, false);
    ///
    /// // Edge-Case
    /// let mut x = B::from(u64::MAX);
    /// let carry = x.add_with_carry(&one);
    /// assert_eq!(x, B::from(0u64));
    /// assert_eq!(carry, true)
    /// ```
    fn add_with_carry(&mut self, other: &Self) -> bool;

    /// Subtract another [`BigInteger`] from this one. This method stores the
    /// result in `self`, and returns a borrow.
    ///
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let (mut one_sub, two, mut three_sub) = (B::from(1u64), B::from(2u64), B::from(3u64));
    /// let borrow = three_sub.sub_with_borrow(&two);
    /// assert_eq!(three_sub, one_sub);
    /// assert_eq!(borrow, false);
    ///
    /// // Edge-Case
    /// let borrow = one_sub.sub_with_borrow(&two);
    /// assert_eq!(one_sub, B::from(u64::MAX));
    /// assert_eq!(borrow, true);
    /// ```
    fn sub_with_borrow(&mut self, other: &Self) -> bool;

    /// Performs a leftwise bitshift of this number, effectively multiplying
    /// it by 2. Overflow is ignored.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let mut two_mul = B::from(2u64);
    /// two_mul.mul2();
    /// assert_eq!(two_mul, B::from(4u64));
    ///
    /// // Edge-Cases
    /// let mut zero = B::from(0u64);
    /// zero.mul2();
    /// assert_eq!(zero, B::from(0u64));
    ///
    /// let mut arr: [bool; 64] = [false; 64];
    /// arr[0] = true;
    /// let mut mul = B::from_bits_be(&arr);
    /// mul.mul2();
    /// assert_eq!(mul, B::from(0u64));
    /// ```
    fn mul2(&mut self) -> bool;

    /// Performs a leftwise bitshift of this number by n bits, effectively
    /// multiplying it by 2^n. Overflow is ignored.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let mut one_mul = B::from(1u64);
    /// one_mul.muln(5);
    /// assert_eq!(one_mul, B::from(32u64));
    ///
    /// // Edge-Case
    /// let mut zero = B::from(0u64);
    /// zero.muln(5);
    /// assert_eq!(zero, B::from(0u64));
    ///
    /// let mut arr: [bool; 64] = [false; 64];
    /// arr[4] = true;
    /// let mut mul = B::from_bits_be(&arr);
    /// mul.muln(5);
    /// assert_eq!(mul, B::from(0u64));
    /// ```
    #[deprecated(
        since = "0.4.2",
        note = "please use the operator `<<` instead"
    )]
    fn muln(&mut self, amt: u32);

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

    /// Performs a rightwise bitshift of this number, effectively dividing
    /// it by 2.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let (mut two, mut four_div) = (B::from(2u64), B::from(4u64));
    /// four_div.div2();
    /// assert_eq!(two, four_div);
    ///
    /// // Edge-Case
    /// let mut zero = B::from(0u64);
    /// zero.div2();
    /// assert_eq!(zero, B::from(0u64));
    ///
    /// let mut one = B::from(1u64);
    /// one.div2();
    /// assert_eq!(one, B::from(0u64));
    /// ```
    fn div2(&mut self);

    /// Performs a rightwise bitshift of this number by some amount.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// // Basic
    /// let (mut one, mut thirty_two_div) = (B::from(1u64), B::from(32u64));
    /// thirty_two_div.divn(5);
    /// assert_eq!(one, thirty_two_div);
    ///
    /// // Edge-Case
    /// let mut arr: [bool; 64] = [false; 64];
    /// arr[4] = true;
    /// let mut div = B::from_bits_le(&arr);
    /// div.divn(5);
    /// assert_eq!(div, B::from(0u64));
    /// ```
    #[deprecated(
        since = "0.4.2",
        note = "please use the operator `>>` instead"
    )]
    fn divn(&mut self, amt: u32);

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
    fn num_bits(&self) -> u32;

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

    /// Returns the big integer representation of a given big endian boolean
    /// array.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let mut arr: [bool; 64] = [false; 64];
    /// arr[63] = true;
    /// let mut one = B::from(1u64);
    /// assert_eq!(B::from_bits_be(&arr), one);
    /// ```
    fn from_bits_be(bits: &[bool]) -> Self;

    /// Returns the big integer representation of a given little endian boolean
    /// array.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let mut arr: [bool; 64] = [false; 64];
    /// arr[0] = true;
    /// let mut one = B::from(1u64);
    /// assert_eq!(B::from_bits_le(&arr), one);
    /// ```
    fn from_bits_le(bits: &[bool]) -> Self;

    // TODO#q: reuse BitIterator
/*    /// Returns the bit representation in a big endian boolean array,
    /// with leading zeroes.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let one = B::from(1u64);
    /// let arr = one.to_bits_be();
    /// let mut vec = vec![false; 64];
    /// vec[63] = true;
    /// assert_eq!(arr, vec);
    /// ```
    fn to_bits_be(&self) -> Vec<bool> {
        BitIteratorBE::new(self).collect::<Vec<_>>()
    }

    /// Returns the bit representation in a little endian boolean array,
    /// with trailing zeroes.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let one = B::from(1u64);
    /// let arr = one.to_bits_le();
    /// let mut vec = vec![false; 64];
    /// vec[0] = true;
    /// assert_eq!(arr, vec);
    /// ```
    fn to_bits_le(&self) -> Vec<bool> {
        BitIteratorLE::new(self).collect::<Vec<_>>()
    }*/

    /// Returns the byte representation in a big endian byte array,
    /// with leading zeros.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let one = B::from(1u64);
    /// let arr = one.to_bytes_be();
    /// let mut vec = vec![0; 8];
    /// vec[7] = 1;
    /// assert_eq!(arr, vec);
    /// ```
    fn to_bytes_be(&self) -> Vec<u8>;

    /// Returns the byte representation in a little endian byte array,
    /// with trailing zeros.
    /// # Example
    ///
    /// ```
    /// use ark_ff::{biginteger::BigInteger64 as B, BigInteger as _};
    ///
    /// let one = B::from(1u64);
    /// let arr = one.to_bytes_le();
    /// let mut vec = vec![0; 8];
    /// vec[0] = 1;
    /// assert_eq!(arr, vec);
    /// ```
    fn to_bytes_le(&self) -> Vec<u8>;
}

impl<const N: usize> BigInteger for crypto_bigint::Uint<N> {
    const NUM_LIMBS: usize = N;

    fn add_with_carry(&mut self, other: &Self) -> bool {
        todo!()
    }

    fn sub_with_borrow(&mut self, other: &Self) -> bool {
        todo!()
    }

    fn mul2(&mut self) -> bool {
        todo!()
    }

    fn muln(&mut self, amt: u32) {
        todo!()
    }

    fn mul_low(&self, other: &Self) -> Self {
        todo!()
    }

    fn mul_high(&self, other: &Self) -> Self {
        todo!()
    }

    fn mul(&self, other: &Self) -> (Self, Self) {
        todo!()
    }

    fn div2(&mut self) {
        todo!()
    }

    fn divn(&mut self, amt: u32) {
        todo!()
    }

    fn is_odd(&self) -> bool {
        todo!()
    }

    fn is_even(&self) -> bool {
        todo!()
    }

    fn is_zero(&self) -> bool {
        todo!()
    }

    fn num_bits(&self) -> u32 {
        todo!()
    }

    fn get_bit(&self, i: usize) -> bool {
        todo!()
    }

    fn from_bits_be(bits: &[bool]) -> Self {
        todo!()
    }

    fn from_bits_le(bits: &[bool]) -> Self {
        todo!()
    }

    // TODO#q: reuse bit iterator
    /*fn to_bits_be(&self) -> Vec<bool> {
        todo!()
    }

    fn to_bits_le(&self) -> Vec<bool> {
        todo!()
    }*/

    fn to_bytes_be(&self) -> Vec<u8> {
        todo!()
    }

    fn to_bytes_le(&self) -> Vec<u8> {
        todo!()
    }
}
