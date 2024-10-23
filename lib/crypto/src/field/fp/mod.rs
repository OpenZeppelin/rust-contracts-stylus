pub mod ops;

use core::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result as FmtResult},
    iter,
    marker::PhantomData,
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
    },
    str::FromStr,
};

use ark_serialize::{
    buffer_byte_size, CanonicalDeserialize, CanonicalDeserializeWithFlags,
    CanonicalSerialize, CanonicalSerializeWithFlags, Compress, EmptyFlags,
    Flags, SerializationError, Valid, Validate,
};
use educe::Educe;
use num_bigint::{BigInt, BigUint};
use num_traits::{FromPrimitive, One, Signed, ToPrimitive, Zero};
use ruint::Uint;

use crate::{
    biginteger::{
        const_ops::{
            const_num_bits, divide_by_2_round_down, two_adic_coefficient,
        },
        BigInteger,
    },
    field::{
        fft::FftField,
        prime::PrimeField,
        sqrt::{LegendreSymbol, SqrtPrecomputation},
        AdditiveGroup, Field,
    },
};

/// A trait that specifies the configuration of a prime field.
/// Also specifies how to perform arithmetic on field elements.
pub trait FpParam<const BITS: usize, const LIMBS: usize>:
    Send + Sync + 'static + Sized
{
    /// The modulus of the field.
    const MODULUS: Uint<BITS, LIMBS>;

    /// A multiplicative generator of the field.
    /// `Self::GENERATOR` is an element having multiplicative order
    /// `Self::MODULUS - 1`.
    const GENERATOR: Fp<Self, BITS, LIMBS>;

    /// Additive identity of the field, i.e. the element `e`
    /// such that, for all elements `f` of the field, `e + f = f`.
    const ZERO: Fp<Self, BITS, LIMBS>;

    /// Multiplicative identity of the field, i.e. the element `e`
    /// such that, for all elements `f` of the field, `e * f = f`.
    const ONE: Fp<Self, BITS, LIMBS>;

    /// Let `N` be the size of the multiplicative group defined by the field.
    /// Then `TWO_ADICITY` is the two-adicity of `N`, i.e. the integer `s`
    /// such that `N = 2^s * t` for some odd integer `t`.
    const TWO_ADICITY: u32;

    /// 2^s root of unity computed by GENERATOR^t
    const TWO_ADIC_ROOT_OF_UNITY: Fp<Self, BITS, LIMBS>;

    /// An integer `b` such that there exists a multiplicative subgroup
    /// of size `b^k` for some integer `k`.
    const SMALL_SUBGROUP_BASE: Option<u32> = None;

    /// The integer `k` such that there exists a multiplicative subgroup
    /// of size `Self::SMALL_SUBGROUP_BASE^k`.
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = None;

    /// GENERATOR^((MODULUS-1) / (2^s *
    /// SMALL_SUBGROUP_BASE^SMALL_SUBGROUP_BASE_ADICITY)) Used for mixed-radix
    /// FFT.
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Fp<Self, BITS, LIMBS>> = None;

    /// Precomputed material for use when computing square roots.
    /// Currently uses the generic Tonelli-Shanks,
    /// which works for every modulus.
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Fp<Self, BITS, LIMBS>>>;

    /// Set a += b.
    fn add_assign(a: &mut Fp<Self, BITS, LIMBS>, b: &Fp<Self, BITS, LIMBS>);

    /// Set a -= b.
    fn sub_assign(a: &mut Fp<Self, BITS, LIMBS>, b: &Fp<Self, BITS, LIMBS>);

    /// Set a = a + a.
    fn double_in_place(a: &mut Fp<Self, BITS, LIMBS>);

    /// Set a = -a;
    fn neg_in_place(a: &mut Fp<Self, BITS, LIMBS>);

    /// Set a *= b.
    fn mul_assign(a: &mut Fp<Self, BITS, LIMBS>, b: &Fp<Self, BITS, LIMBS>);

    /// Compute the inner product `<a, b>`.
    fn sum_of_products<const T: usize>(
        a: &[Fp<Self, BITS, LIMBS>; T],
        b: &[Fp<Self, BITS, LIMBS>; T],
    ) -> Fp<Self, BITS, LIMBS>;

    /// Set a *= a.
    fn square_in_place(a: &mut Fp<Self, BITS, LIMBS>);

    /// Compute a^{-1} if `a` is not zero.
    fn inverse(a: &Fp<Self, BITS, LIMBS>) -> Option<Fp<Self, BITS, LIMBS>>;

    /// Construct a field element from an integer in the range
    /// `0..(Self::MODULUS - 1)`. Returns `None` if the integer is outside
    /// this range.
    fn from_bigint(other: Uint<BITS, LIMBS>) -> Option<Fp<Self, BITS, LIMBS>>;

    /// Convert a field element to an integer in the range `0..(Self::MODULUS -
    /// 1)`.
    fn into_bigint(other: Fp<Self, BITS, LIMBS>) -> Uint<BITS, LIMBS>;
}

/// Represents an element of the prime field F_p, where `p == P::MODULUS`.
/// This type can represent elements in any field of size at most N * 64 bits.
#[derive(Educe)]
#[educe(Default, Hash, Clone, Copy, PartialEq, Eq)]
pub struct Fp<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>(
    /// Contains the element in Montgomery form for efficient multiplication.
    /// To convert an element to a [`BigInt`](struct@BigInt), use `into_bigint`
    /// or `into`.
    #[doc(hidden)]
    pub Uint<BITS, LIMBS>,
    #[doc(hidden)] pub PhantomData<P>,
);

// pub type Fp64<P> = Fp<P, 1>;
// pub type Fp128<P> = Fp<P, 2>;
// pub type Fp192<P> = Fp<P, 3>;
// pub type Fp256<P> = Fp<P, 4>;
// pub type Fp320<P> = Fp<P, 5>;
// pub type Fp384<P> = Fp<P, 6>;
// pub type Fp448<P> = Fp<P, 7>;
// pub type Fp512<P> = Fp<P, 8>;
// pub type Fp576<P> = Fp<P, 9>;
// pub type Fp640<P> = Fp<P, 10>;
// pub type Fp704<P> = Fp<P, 11>;
// pub type Fp768<P> = Fp<P, 12>;
// pub type Fp832<P> = Fp<P, 13>;

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    Fp<P, BITS, LIMBS>
{
    #[doc(hidden)]
    #[inline]
    pub fn is_geq_modulus(&self) -> bool {
        self.0 >= P::MODULUS
    }

    #[inline]
    fn subtract_modulus(&mut self) {
        if self.is_geq_modulus() {
            self.0.sub_with_borrow(&Self::MODULUS);
        }
    }

    #[inline]
    fn subtract_modulus_with_carry(&mut self, carry: bool) {
        if carry || self.is_geq_modulus() {
            self.0.sub_with_borrow(&Self::MODULUS);
        }
    }

    fn num_bits_to_shave() -> usize {
        BITS - (Self::MODULUS_BIT_SIZE as usize)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    ark_std::fmt::Debug for Fp<P, BITS, LIMBS>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> ark_std::fmt::Result {
        ark_std::fmt::Debug::fmt(&self.into_bigint(), f)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> Zero
    for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn zero() -> Self {
        P::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == P::ZERO
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> One
    for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn one() -> Self {
        P::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        *self == P::ONE
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    AdditiveGroup for Fp<P, BITS, LIMBS>
{
    type Scalar = Self;

    const ZERO: Self = P::ZERO;

    #[inline]
    fn double(&self) -> Self {
        let mut temp = *self;
        temp.double_in_place();
        temp
    }

    #[inline]
    fn double_in_place(&mut self) -> &mut Self {
        P::double_in_place(self);
        self
    }

    #[inline]
    fn neg_in_place(&mut self) -> &mut Self {
        P::neg_in_place(self);
        self
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> Field
    for Fp<P, BITS, LIMBS>
{
    type BasePrimeField = Self;

    const ONE: Self = P::ONE;
    const SQRT_PRECOMP: Option<SqrtPrecomputation<Self>> = P::SQRT_PRECOMP;

    fn extension_degree() -> u64 {
        1
    }

    fn from_base_prime_field(elem: Self::BasePrimeField) -> Self {
        elem
    }

    fn to_base_prime_field_elements(
        &self,
    ) -> impl Iterator<Item = Self::BasePrimeField> {
        iter::once(*self)
    }

    fn from_base_prime_field_elems(
        elems: impl IntoIterator<Item = Self::BasePrimeField>,
    ) -> Option<Self> {
        let mut elems = elems.into_iter();
        let elem = elems.next()?;
        if elems.next().is_some() {
            return None;
        }
        Some(elem)
    }

    #[inline]
    fn characteristic() -> &'static [u64] {
        P::MODULUS.as_limbs()
    }

    #[inline]
    fn sum_of_products<const T: usize>(a: &[Self; T], b: &[Self; T]) -> Self {
        P::sum_of_products(a, b)
    }

    #[inline]
    fn from_random_bytes_with_flags<F: Flags>(
        bytes: &[u8],
    ) -> Option<(Self, F)> {
        if F::BIT_SIZE > 8 {
            None
        } else {
            let shave_bits = Self::num_bits_to_shave();
            let mut result_bytes =
                crate::const_helpers::SerBuffer::<LIMBS>::zeroed();
            // Copy the input into a temporary buffer.
            result_bytes.copy_from_u8_slice(bytes);
            // This mask retains everything in the last limb
            // that is below `P::MODULUS_BIT_SIZE`.
            let last_limb_mask =
                (u64::MAX.checked_shr(shave_bits as u32).unwrap_or(0))
                    .to_le_bytes();
            let mut last_bytes_mask = [0u8; 9];
            last_bytes_mask[..8].copy_from_slice(&last_limb_mask);

            // Length of the buffer containing the field element and the flag.
            let output_byte_size =
                buffer_byte_size(Self::MODULUS_BIT_SIZE as usize + F::BIT_SIZE);
            // Location of the flag is the last byte of the serialized
            // form of the field element.
            let flag_location = output_byte_size - 1;

            // At which byte is the flag located in the last limb?
            let flag_location_in_last_limb =
                flag_location.saturating_sub(8 * (LIMBS - 1));

            // Take all but the last 9 bytes.
            let last_bytes = result_bytes.last_n_plus_1_bytes_mut();

            // The mask only has the last `F::BIT_SIZE` bits set
            let flags_mask =
                u8::MAX.checked_shl(8 - (F::BIT_SIZE as u32)).unwrap_or(0);

            // Mask away the remaining bytes, and try to reconstruct the
            // flag
            let mut flags: u8 = 0;
            for (i, (b, m)) in last_bytes.zip(&last_bytes_mask).enumerate() {
                if i == flag_location_in_last_limb {
                    flags = *b & flags_mask
                }
                *b &= m;
            }
            Self::deserialize_compressed(
                &result_bytes.as_slice()[..(LIMBS * 8)],
            )
            .ok()
            .and_then(|f| F::from_u8(flags).map(|flag| (f, flag)))
        }
    }

    #[inline]
    fn square(&self) -> Self {
        let mut temp = *self;
        temp.square_in_place();
        temp
    }

    fn square_in_place(&mut self) -> &mut Self {
        P::square_in_place(self);
        self
    }

    #[inline]
    fn inverse(&self) -> Option<Self> {
        P::inverse(self)
    }

    fn inverse_in_place(&mut self) -> Option<&mut Self> {
        if let Some(inverse) = self.inverse() {
            *self = inverse;
            Some(self)
        } else {
            None
        }
    }

    /// The Frobenius map has no effect in a prime field.
    #[inline]
    fn frobenius_map_in_place(&mut self, _: usize) {}

    #[inline]
    fn legendre(&self) -> LegendreSymbol {
        // s = self^((MODULUS - 1) // 2)
        let s = self.pow(Self::MODULUS_MINUS_ONE_DIV_TWO);
        if s.is_zero() {
            LegendreSymbol::Zero
        } else if s.is_one() {
            LegendreSymbol::QuadraticResidue
        } else {
            LegendreSymbol::QuadraticNonResidue
        }
    }

    /// Fp is already a "BasePrimeField", so it's just mul by self
    #[inline]
    fn mul_by_base_prime_field(&self, elem: &Self::BasePrimeField) -> Self {
        *self * elem
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> PrimeField
    for Fp<P, BITS, LIMBS>
{
    type BigInt = Uint<BITS, LIMBS>;

    const MODULUS: Self::BigInt = P::MODULUS;
    const MODULUS_BIT_SIZE: u32 = const_num_bits(P::MODULUS);
    const MODULUS_MINUS_ONE_DIV_TWO: Self::BigInt =
        divide_by_2_round_down(P::MODULUS);
    const TRACE: Self::BigInt = two_adic_coefficient(P::MODULUS);
    const TRACE_MINUS_ONE_DIV_TWO: Self::BigInt =
        divide_by_2_round_down(Self::TRACE);

    #[inline]
    fn from_bigint(r: Uint<BITS, LIMBS>) -> Option<Self> {
        P::from_bigint(r)
    }

    fn into_bigint(self) -> Uint<BITS, LIMBS> {
        P::into_bigint(self)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> FftField
    for Fp<P, BITS, LIMBS>
{
    const GENERATOR: Self = P::GENERATOR;
    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> =
        P::LARGE_SUBGROUP_ROOT_OF_UNITY;
    const SMALL_SUBGROUP_BASE: Option<u32> = P::SMALL_SUBGROUP_BASE;
    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> =
        P::SMALL_SUBGROUP_BASE_ADICITY;
    const TWO_ADICITY: u32 = P::TWO_ADICITY;
    const TWO_ADIC_ROOT_OF_UNITY: Self = P::TWO_ADIC_ROOT_OF_UNITY;
}

/// Note that this implementation of `Ord` compares field elements viewing
/// them as integers in the range 0, 1, ..., P::MODULUS - 1. However, other
/// implementations of `PrimeField` might choose a different ordering, and
/// as such, users should use this `Ord` for applications where
/// any ordering suffices (like in a BTreeMap), and not in applications
/// where a particular ordering is required.
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> Ord
    for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_bigint().cmp(&other.into_bigint())
    }
}

/// Note that this implementation of `PartialOrd` compares field elements
/// viewing them as integers in the range 0, 1, ..., `P::MODULUS` - 1. However,
/// other implementations of `PrimeField` might choose a different ordering, and
/// as such, users should use this `PartialOrd` for applications where
/// any ordering suffices (like in a BTreeMap), and not in applications
/// where a particular ordering is required.
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> PartialOrd
    for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<u128>
    for Fp<P, BITS, LIMBS>
{
    fn from(mut other: u128) -> Self {
        Uint::from_u128(other)
            .unwrap_or_else(|| {
                let modulus = P::MODULUS.to_u128().unwrap();

                other %= modulus;
                Uint::from_u128(other).unwrap()
            })
            .into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<i128>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: i128) -> Self {
        let other_abs = other.unsigned_abs();
        let fp = other_abs.into();

        if other.is_positive() {
            fp
        } else {
            -fp
        }
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<bool>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: bool) -> Self {
        let other = other as i8;
        other.into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<u64>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: u64) -> Self {
        Uint::from_u64(other)
            .unwrap_or_else(|| {
                let modulus = P::MODULUS.to_u64().unwrap();

                Uint::from_u64(other % modulus).unwrap()
            })
            .into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<i64>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: i64) -> Self {
        let abs = Self::from(other.unsigned_abs());

        if other.is_positive() {
            abs
        } else {
            -abs
        }
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<u32>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: u32) -> Self {
        Uint::from_u32(other)
            .unwrap_or_else(|| {
                let modulus = P::MODULUS.to_u32().unwrap();

                Uint::from_u32(other % modulus).unwrap()
            })
            .into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<i32>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: i32) -> Self {
        let other_abs = other.unsigned_abs();
        let fp = other_abs.into();

        if other.is_positive() {
            fp
        } else {
            -fp
        }
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<u16>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: u16) -> Self {
        Uint::from_u16(other)
            .unwrap_or_else(|| {
                let modulus = P::MODULUS.to_u16().unwrap();

                Uint::from_u16(other % modulus).unwrap()
            })
            .into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<i16>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: i16) -> Self {
        let other_abs = other.unsigned_abs();
        let fp = other_abs.into();

        if other.is_positive() {
            fp
        } else {
            -fp
        }
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<u8>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: u8) -> Self {
        Uint::from_u8(other)
            .unwrap_or_else(|| {
                let modulus = P::MODULUS.to_u8().unwrap();

                Uint::from_u8(other % modulus).unwrap()
            })
            .into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> From<i8>
    for Fp<P, BITS, LIMBS>
{
    fn from(other: i8) -> Self {
        let other_abs = other.unsigned_abs();
        let fp = other_abs.into();

        if other.is_positive() {
            fp
        } else {
            -fp
        }
    }
}

// TODO#q: random for Fp
/*
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    ark_std::rand::distributions::Distribution<Fp<P, BITS, LIMBS>>
    for ark_std::rand::distributions::Standard
{
    #[inline]
    fn sample<R: ark_std::rand::Rng + ?Sized>(
        &self,
        rng: &mut R,
    ) -> Fp<P, BITS, LIMBS> {
        loop {
            let mut tmp = Fp(
                rng.sample(ark_std::rand::distributions::Standard),
                PhantomData,
            );
            let shave_bits = Fp::<P, BITS, LIMBS>::num_bits_to_shave();
            // Mask away the unused bits at the beginning.
            assert!(shave_bits <= 64);
            let mask =
                if shave_bits == 64 { 0 } else { u64::MAX >> shave_bits };

            if let Some(val) = tmp.0 .0.last_mut() {
                *val &= mask
            }

            if !tmp.is_geq_modulus() {
                return tmp;
            }
        }
    }
}
*/

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    CanonicalSerializeWithFlags for Fp<P, BITS, LIMBS>
{
    fn serialize_with_flags<W: ark_std::io::Write, F: Flags>(
        &self,
        writer: W,
        flags: F,
    ) -> Result<(), SerializationError> {
        // All reasonable `Flags` should be less than 8 bits in size
        // (256 values are enough for anyone!)
        if F::BIT_SIZE > 8 {
            return Err(SerializationError::NotEnoughSpace);
        }

        // Calculate the number of bytes required to represent a field element
        // serialized with `flags`. If `F::BIT_SIZE < 8`,
        // this is at most `N * 8 + 1`
        let output_byte_size =
            buffer_byte_size(Self::MODULUS_BIT_SIZE as usize + F::BIT_SIZE);

        // Write out `self` to a temporary buffer.
        // The size of the buffer is $byte_size + 1 because `F::BIT_SIZE`
        // is at most 8 bits.
        let mut bytes = crate::const_helpers::SerBuffer::zeroed();
        bytes.copy_from_u64_slice(&self.into_bigint().as_limbs());
        // Mask out the bits of the last byte that correspond to the flag.
        bytes[output_byte_size - 1] |= flags.u8_bitmask();

        bytes.write_up_to(writer, output_byte_size)?;
        Ok(())
    }

    // Let `m = 8 * n` for some `n` be the smallest multiple of 8 greater
    // than `P::MODULUS_BIT_SIZE`.
    // If `(m - P::MODULUS_BIT_SIZE) >= F::BIT_SIZE` , then this method returns
    // `n`; otherwise, it returns `n + 1`.
    fn serialized_size_with_flags<F: Flags>(&self) -> usize {
        buffer_byte_size(Self::MODULUS_BIT_SIZE as usize + F::BIT_SIZE)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    CanonicalSerialize for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn serialize_with_mode<W: ark_std::io::Write>(
        &self,
        writer: W,
        _compress: Compress,
    ) -> Result<(), SerializationError> {
        self.serialize_with_flags(writer, EmptyFlags)
    }

    #[inline]
    fn serialized_size(&self, _compress: Compress) -> usize {
        self.serialized_size_with_flags::<EmptyFlags>()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    CanonicalDeserializeWithFlags for Fp<P, BITS, LIMBS>
{
    fn deserialize_with_flags<R: ark_std::io::Read, F: Flags>(
        reader: R,
    ) -> Result<(Self, F), SerializationError> {
        // All reasonable `Flags` should be less than 8 bits in size
        // (256 values are enough for anyone!)
        if F::BIT_SIZE > 8 {
            return Err(SerializationError::NotEnoughSpace);
        }
        // Calculate the number of bytes required to represent a field element
        // serialized with `flags`.
        let output_byte_size = Self::zero().serialized_size_with_flags::<F>();

        let mut masked_bytes = crate::const_helpers::SerBuffer::zeroed();
        masked_bytes.read_exact_up_to(reader, output_byte_size)?;
        let flags =
            F::from_u8_remove_flags(&mut masked_bytes[output_byte_size - 1])
                .ok_or(SerializationError::UnexpectedFlags)?;

        let self_integer = masked_bytes.to_bigint();
        Self::from_bigint(self_integer)
            .map(|v| (v, flags))
            .ok_or(SerializationError::InvalidData)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> Valid
    for Fp<P, BITS, LIMBS>
{
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    CanonicalDeserialize for Fp<P, BITS, LIMBS>
{
    fn deserialize_with_mode<R: ark_std::io::Read>(
        reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        Self::deserialize_with_flags::<R, EmptyFlags>(reader).map(|(r, _)| r)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> FromStr
    for Fp<P, BITS, LIMBS>
{
    type Err = ();

    /// Interpret a string of numbers as a (congruent) prime field element.
    /// Does not accept unnecessary leading zeroes or a blank string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let modulus = BigInt::from(P::MODULUS);
        let mut a = BigInt::from_str(s).map_err(|_| ())? % &modulus;
        if a.is_negative() {
            a += modulus
        }
        let biguint = BigUint::try_from(a).expect("biguint");
        let uint = Uint::<BITS, LIMBS>::try_from(&biguint).expect("uint");
        Self::from_bigint(uint).ok_or(())
    }
}

/// Outputs a string containing the value of `self`,
/// represented as a decimal without leading zeroes.
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> Display
    for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let string = self.into_bigint().to_string();
        write!(f, "{}", string)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize> Neg
    for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(mut self) -> Self {
        P::neg_in_place(&mut self);
        self
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    Add<&'a Fp<P, BITS, LIMBS>> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &Self) -> Self {
        self.add_assign(other);
        self
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    Sub<&'a Fp<P, BITS, LIMBS>> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &Self) -> Self {
        self.sub_assign(other);
        self
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    Mul<&'a Fp<P, BITS, LIMBS>> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &Self) -> Self {
        self.mul_assign(other);
        self
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    Div<&'a Fp<P, BITS, LIMBS>> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    /// Returns `self * other.inverse()` if `other.inverse()` is `Some`, and
    /// panics otherwise.
    #[inline]
    fn div(mut self, other: &Self) -> Self {
        self.mul_assign(&other.inverse().unwrap());
        self
    }
}

impl<
        'a,
        'b,
        P: FpParam<BITS, LIMBS>,
        const BITS: usize,
        const LIMBS: usize,
    > Add<&'b Fp<P, BITS, LIMBS>> for &'a Fp<P, BITS, LIMBS>
{
    type Output = Fp<P, BITS, LIMBS>;

    #[inline]
    fn add(self, other: &'b Fp<P, BITS, LIMBS>) -> Fp<P, BITS, LIMBS> {
        let mut result = *self;
        result.add_assign(other);
        result
    }
}

impl<
        'a,
        'b,
        P: FpParam<BITS, LIMBS>,
        const BITS: usize,
        const LIMBS: usize,
    > Sub<&'b Fp<P, BITS, LIMBS>> for &'a Fp<P, BITS, LIMBS>
{
    type Output = Fp<P, BITS, LIMBS>;

    #[inline]
    fn sub(self, other: &Fp<P, BITS, LIMBS>) -> Fp<P, BITS, LIMBS> {
        let mut result = *self;
        result.sub_assign(other);
        result
    }
}

impl<
        'a,
        'b,
        P: FpParam<BITS, LIMBS>,
        const BITS: usize,
        const LIMBS: usize,
    > Mul<&'b Fp<P, BITS, LIMBS>> for &'a Fp<P, BITS, LIMBS>
{
    type Output = Fp<P, BITS, LIMBS>;

    #[inline]
    fn mul(self, other: &Fp<P, BITS, LIMBS>) -> Fp<P, BITS, LIMBS> {
        let mut result = *self;
        result.mul_assign(other);
        result
    }
}

impl<
        'a,
        'b,
        P: FpParam<BITS, LIMBS>,
        const BITS: usize,
        const LIMBS: usize,
    > Div<&'b Fp<P, BITS, LIMBS>> for &'a Fp<P, BITS, LIMBS>
{
    type Output = Fp<P, BITS, LIMBS>;

    #[inline]
    fn div(self, other: &Fp<P, BITS, LIMBS>) -> Fp<P, BITS, LIMBS> {
        let mut result = *self;
        result.div_assign(other);
        result
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    AddAssign<&'a Self> for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        P::add_assign(self, other)
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    SubAssign<&'a Self> for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn sub_assign(&mut self, other: &Self) {
        P::sub_assign(self, other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Add<Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: Self) -> Self {
        self.add_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Add<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &'a mut Self) -> Self {
        self.add_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Sub<Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: Self) -> Self {
        self.sub_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Sub<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &'a mut Self) -> Self {
        self.sub_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::iter::Sum<Self> for Fp<P, BITS, LIMBS>
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::iter::Sum<&'a Self> for Fp<P, BITS, LIMBS>
{
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::AddAssign<Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::SubAssign<Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::AddAssign<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn add_assign(&mut self, other: &'a mut Self) {
        self.add_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::SubAssign<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn sub_assign(&mut self, other: &'a mut Self) {
        self.sub_assign(&*other)
    }
}

impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    MulAssign<&'a Self> for Fp<P, BITS, LIMBS>
{
    fn mul_assign(&mut self, other: &Self) {
        P::mul_assign(self, other)
    }
}

/// Computes `self *= other.inverse()` if `other.inverse()` is `Some`, and
/// panics otherwise.
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    DivAssign<&'a Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn div_assign(&mut self, other: &Self) {
        self.mul_assign(&other.inverse().unwrap());
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Mul<Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        self.mul_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Div<Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: Self) -> Self {
        self.div_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Mul<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: &'a mut Self) -> Self {
        self.mul_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::Div<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: &'a mut Self) -> Self {
        self.div_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::iter::Product<Self> for Fp<P, BITS, LIMBS>
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::iter::Product<&'a Self> for Fp<P, BITS, LIMBS>
{
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::MulAssign<Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        self.mul_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::DivAssign<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn div_assign(&mut self, other: &'a mut Self) {
        self.div_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::MulAssign<&'a mut Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn mul_assign(&mut self, other: &'a mut Self) {
        self.mul_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    core::ops::DivAssign<Self> for Fp<P, BITS, LIMBS>
{
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        self.div_assign(&other)
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    zeroize::Zeroize for Fp<P, BITS, LIMBS>
{
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    From<BigUint> for Fp<P, BITS, LIMBS>
{
    #[inline]
    fn from(val: BigUint) -> Fp<P, BITS, LIMBS> {
        Fp::<P, BITS, LIMBS>::from_le_bytes_mod_order(&val.to_bytes_le())
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    From<Fp<P, BITS, LIMBS>> for BigUint
{
    #[inline(always)]
    fn from(other: Fp<P, BITS, LIMBS>) -> Self {
        other.into_bigint().into()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    From<Fp<P, BITS, LIMBS>> for Uint<BITS, LIMBS>
{
    #[inline(always)]
    fn from(fp: Fp<P, BITS, LIMBS>) -> Self {
        fp.into_bigint()
    }
}

impl<P: FpParam<BITS, LIMBS>, const BITS: usize, const LIMBS: usize>
    From<Uint<BITS, LIMBS>> for Fp<P, BITS, LIMBS>
{
    /// Converts `Self::BigInteger` into `Self`
    #[inline(always)]
    fn from(int: Uint<BITS, LIMBS>) -> Self {
        Self::from_bigint(int).unwrap()
    }
}
