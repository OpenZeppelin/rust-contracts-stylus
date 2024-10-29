use crate::{biginteger::BigInteger, field::Field};

/// The interface for a prime field, i.e. the field of integers modulo a prime
/// $p$. In the following example we'll use the prime field underlying the
/// BLS12-381 G1 curve.
/// ```rust
/// use ark_ff::{BigInteger, Field, PrimeField, Zero};
/// use ark_std::{test_rng, One, UniformRand};
/// use ark_test_curves::bls12_381::Fq as F;
///
/// let mut rng = test_rng();
/// let a = F::rand(&mut rng);
/// // We can access the prime modulus associated with `F`:
/// let modulus = <F as PrimeField>::MODULUS;
/// assert_eq!(a.pow(&modulus), a); // the Euler-Fermat theorem tells us:
/// a^{p-1} = 1 mod p
///
/// // We can convert field elements to integers in the range [0, MODULUS - 1]:
/// let one: num_bigint::BigUint = F::one().into();
/// assert_eq!(one, num_bigint::BigUint::one());
///
/// // We can construct field elements from an arbitrary sequence of bytes:
/// let n = F::from_le_bytes_mod_order(&modulus.to_bytes_le());
/// assert_eq!(n, F::zero());
/// ```
pub trait PrimeField:
    Field + From<<Self as PrimeField>::BigInt> + Into<<Self as PrimeField>::BigInt>
{
    /// A `BigInteger` type that can represent elements of this field.
    type BigInt: BigInteger;

    /// The modulus `p`.
    const MODULUS: Self::BigInt;

    /// The size of the modulus in bits.
    const MODULUS_BIT_SIZE: usize;

    /// Construct a prime field element from an integer in the range 0..(p - 1).
    fn from_bigint(repr: Self::BigInt) -> Option<Self>;

    /// Converts an element of the prime field into an integer in the range
    /// 0..(p - 1).
    fn into_bigint(self) -> Self::BigInt;
}
