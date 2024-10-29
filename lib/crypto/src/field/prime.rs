use crate::{biginteger::BigInteger, field::Field};

/// Defines an abstract prime field.
/// I.e., the field of integers of prime module [`Self::MODULUS`].
pub trait PrimeField:
    Field + From<<Self as PrimeField>::BigInt> + Into<<Self as PrimeField>::BigInt>
{
    /// A `BigInteger` type that can represent elements of this field.
    type BigInt: BigInteger;

    /// The modulus `p`.
    const MODULUS: Self::BigInt;

    /// The size of the modulus in bits.
    const MODULUS_BIT_SIZE: usize;

    /// Construct a prime field element from an integer in the range `0..(p -
    /// 1)`.
    fn from_bigint(repr: Self::BigInt) -> Option<Self>;

    /// Converts an element of the prime field into an integer in the range
    /// `0..(p - 1)`.
    fn into_bigint(self) -> Self::BigInt;
}
