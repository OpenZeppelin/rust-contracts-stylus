use core::{
    fmt::{Debug, Display},
    hash::Hash,
    iter::{Product, Sum},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
    },
};

use ark_serialize::{
    CanonicalDeserialize, CanonicalDeserializeWithFlags, CanonicalSerialize,
    CanonicalSerializeWithFlags, EmptyFlags, Flags,
};
use ark_std::UniformRand;
use num_traits::{One, Zero};
use zeroize::Zeroize;

use crate::{
    bits::{BitIteratorBE, BitIteratorLE},
    field::{
        prime::PrimeField,
        sqrt::{LegendreSymbol, SqrtPrecomputation},
    },
};

pub mod fft_friendly;
pub mod fp;
pub mod merkle_tree_fp;
pub mod prime;
pub mod sqrt;
pub mod utils;
pub mod vesta;

/// The interface for a generic field.
/// Types implementing [`Field`] support common field operations such as
/// addition, subtraction, multiplication, and inverses.
pub trait Field:
    'static
    + Copy
    + Clone
    + Debug
    + Display
    + Default
    + Send
    + Sync
    + Eq
    + Zero
    + One
    + Ord
    + Neg<Output = Self>
    + UniformRand
    + Zeroize
    + Sized
    + Hash
    + CanonicalSerialize
    + CanonicalSerializeWithFlags
    + CanonicalDeserialize
    + CanonicalDeserializeWithFlags
    + AdditiveGroup<Scalar = Self>
    + Div<Self, Output = Self>
    + DivAssign<Self>
    + for<'a> Div<&'a Self, Output = Self>
    + for<'a> DivAssign<&'a Self>
    + for<'a> Div<&'a mut Self, Output = Self>
    + for<'a> DivAssign<&'a mut Self>
    + for<'a> core::iter::Product<&'a Self>
    + From<u128>
    + From<u64>
    + From<u32>
    + From<u16>
    + From<u8>
    + From<i128>
    + From<i64>
    + From<i32>
    + From<i16>
    + From<i8>
    + From<bool>
    + Product<Self>
{
    type BasePrimeField: PrimeField;

    /// Determines the algorithm for computing square roots.
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Self>>;

    /// The multiplicative identity of the field.
    const ONE: Self;

    /// Returns the characteristic of the field,
    /// in little-endian representation.
    fn characteristic() -> &'static [u64] {
        Self::BasePrimeField::characteristic()
    }

    /// Returns the extension degree of this field with respect
    /// to `Self::BasePrimeField`.
    fn extension_degree() -> u64;

    fn to_base_prime_field_elements(
        &self,
    ) -> impl Iterator<Item = Self::BasePrimeField>;

    /// Convert a slice of base prime field elements into a field element.
    /// If the slice length != Self::extension_degree(), must return None.
    fn from_base_prime_field_elems(
        elems: impl IntoIterator<Item = Self::BasePrimeField>,
    ) -> Option<Self>;

    /// Constructs a field element from a single base prime field elements.
    fn from_base_prime_field(elem: Self::BasePrimeField) -> Self;

    // TODO#q: add random bytes conversions

    /// Attempt to deserialize a field element. Returns `None` if the
    /// deserialization fails.
    ///
    /// This function is primarily intended for sampling random field elements
    /// from a hash-function or RNG output.
    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        Self::from_random_bytes_with_flags::<EmptyFlags>(bytes).map(|f| f.0)
    }

    /// Attempt to deserialize a field element, splitting the bitflags metadata
    /// according to `F` specification. Returns `None` if the deserialization
    /// fails.
    ///
    /// This function is primarily intended for sampling random field elements
    /// from a hash-function or RNG output.
    fn from_random_bytes_with_flags<F: Flags>(
        bytes: &[u8],
    ) -> Option<(Self, F)>;

    /// Returns a `LegendreSymbol`, which indicates whether this field element
    /// is  1 : a quadratic residue
    ///  0 : equal to 0
    /// -1 : a quadratic non-residue
    fn legendre(&self) -> LegendreSymbol;

    /// Returns the square root of self, if it exists.
    #[must_use]
    fn sqrt(&self) -> Option<Self> {
        match Self::SQRT_PRECOMP {
            Some(tv) => tv.sqrt(self),
            None => unimplemented!(),
        }
    }

    /// Sets `self` to be the square root of `self`, if it exists.
    fn sqrt_in_place(&mut self) -> Option<&mut Self> {
        (*self).sqrt().map(|sqrt| {
            *self = sqrt;
            self
        })
    }

    /// Returns `self * self`.
    #[must_use]
    fn square(&self) -> Self;

    /// Squares `self` in place.
    fn square_in_place(&mut self) -> &mut Self;

    /// Computes the multiplicative inverse of `self` if `self` is nonzero.
    #[must_use]
    fn inverse(&self) -> Option<Self>;

    /// If `self.inverse().is_none()`, this just returns `None`. Otherwise, it
    /// sets `self` to `self.inverse().unwrap()`.
    fn inverse_in_place(&mut self) -> Option<&mut Self>;

    /// Returns `sum([a_i * b_i])`.
    #[inline]
    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        let mut sum = Self::zero();
        for i in 0..a.len() {
            sum += a[i] * b[i];
        }
        sum
    }

    /// Sets `self` to `self^s`, where `s =
    /// Self::BasePrimeField::MODULUS^power`. This is also called the
    /// Frobenius automorphism.
    fn frobenius_map_in_place(&mut self, power: usize);

    /// Returns `self^s`, where `s = Self::BasePrimeField::MODULUS^power`.
    /// This is also called the Frobenius automorphism.
    #[must_use]
    fn frobenius_map(&self, power: usize) -> Self {
        let mut this = *self;
        this.frobenius_map_in_place(power);
        this
    }

    /// Returns `self^exp`, where `exp` is an integer represented with `u64`
    /// limbs, least significant limb first.
    #[must_use]
    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::one();

        for i in BitIteratorBE::without_leading_zeros(exp) {
            res.square_in_place();

            if i {
                res *= self;
            }
        }
        res
    }

    /// Exponentiates a field element `f` by a number represented with `u64`
    /// limbs, using a precomputed table containing as many powers of 2 of
    /// `f` as the 1 + the floor of log2 of the exponent `exp`, starting
    /// from the 1st power. That is, `powers_of_2` should equal `&[p, p^2,
    /// p^4, ..., p^(2^n)]` when `exp` has at most `n` bits.
    ///
    /// This returns `None` when a power is missing from the table.
    #[inline]
    fn pow_with_table<S: AsRef<[u64]>>(
        powers_of_2: &[Self],
        exp: S,
    ) -> Option<Self> {
        let mut res = Self::one();
        for (pow, bit) in BitIteratorLE::without_trailing_zeros(exp).enumerate()
        {
            if bit {
                res *= powers_of_2.get(pow)?;
            }
        }
        Some(res)
    }

    fn mul_by_base_prime_field(&self, elem: &Self::BasePrimeField) -> Self;
}

pub trait AdditiveGroup:
    Eq
    + 'static
    + Sized
    // + CanonicalSerialize // TODO#q add this serialisation
    // + CanonicalDeserialize
    + Copy
    + Clone
    + Default
    + Send
    + Sync
    + Hash
    + Debug
    + Display
    // + UniformRand // TODO#q add uniform rand generation
    + Zeroize
    + Zero
    + Neg<Output = Self>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<<Self as AdditiveGroup>::Scalar, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<<Self as AdditiveGroup>::Scalar>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a <Self as AdditiveGroup>::Scalar, Output = Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a <Self as AdditiveGroup>::Scalar>
    + for<'a> Add<&'a mut Self, Output = Self>
    + for<'a> Sub<&'a mut Self, Output = Self>
    + for<'a> Mul<&'a mut <Self as AdditiveGroup>::Scalar, Output = Self>
    + for<'a> AddAssign<&'a mut Self>
    + for<'a> SubAssign<&'a mut Self>
    + for<'a> MulAssign<&'a mut <Self as AdditiveGroup>::Scalar>
    + Sum<Self>
    + for<'a> Sum<&'a Self>
{
    type Scalar: Field;

    /// The additive identity of the field.
    const ZERO: Self;

    /// Doubles `self`.
    #[must_use]
    fn double(&self) -> Self {
        let mut copy = *self;
        copy.double_in_place();
        copy
    }
    /// Doubles `self` in place.
    fn double_in_place(&mut self) -> &mut Self {
        self.add_assign(*self);
        self
    }

    /// Negates `self` in place.
    fn neg_in_place(&mut self) -> &mut Self {
        *self = -(*self);
        self
    }
}
