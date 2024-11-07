//! This module contains the implementation of a prime field element [`Fp`],
//! altogether with exact implementations [`Fp64`] for 64-bit, [`Fp128`] for
//! 128-bit elements and so on.
//!
//! Finite field element [`Fp`] wraps a biginteger element in [motgomery form],
//! which is used for efficient multiplication and division.
//!
//! Note that implementation of `Ord` for [`Fp`] compares field elements viewing
//! them as integers in the range `0, 1, ..., P::MODULUS - 1`.
//! However, other implementations of `PrimeField` might choose a different
//! ordering, and as such, users should use this `Ord` for applications where
//! any ordering suffices (like in a `BTreeMap`), and not in applications
//! where a particular ordering is required.
//!
//! [motgomery form]: https://en.wikipedia.org/wiki/Montgomery_modular_multiplication
use alloc::string::ToString;
use core::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crypto_bigint::{
    modular::{
        constant_mod::{Residue, ResidueParams},
        montgomery_reduction,
    },
    Limb, Uint, Word,
};
use educe::Educe;
use num_traits::{One, Zero};

use crate::field::{group::AdditiveGroup, prime::PrimeField, Field};

/// A trait that specifies the configuration of a prime field.
/// Also specifies how to perform arithmetic on field elements.
pub trait FpParams<const LIMBS: usize>: Send + Sync + 'static + Sized {
    /// The modulus of the field.
    const MODULUS: Uint<LIMBS>;

    /// A multiplicative generator of the field.
    /// [`Self::GENERATOR`] is an element having multiplicative order
    /// `MODULUS - 1`.
    const GENERATOR: Fp<Self, LIMBS>;

    /// Set `a += b`.
    fn add_assign(a: &mut Fp<Self, LIMBS>, b: &Fp<Self, LIMBS>) {
        a.residue += b.residue;
    }

    /// Set `a -= b`.
    fn sub_assign(a: &mut Fp<Self, LIMBS>, b: &Fp<Self, LIMBS>) {
        a.residue -= b.residue;
    }

    /// Set `a = a + a`.
    fn double_in_place(a: &mut Fp<Self, LIMBS>) {
        a.residue += a.residue;
    }

    /// Set `a = -a`;
    fn neg_in_place(a: &mut Fp<Self, LIMBS>) {
        a.residue = a.residue.neg();
    }

    /// Set `a *= b`.
    fn mul_assign(a: &mut Fp<Self, LIMBS>, b: &Fp<Self, LIMBS>) {
        a.residue *= b.residue;
    }

    /// Set `a *= a`.
    fn square_in_place(a: &mut Fp<Self, LIMBS>) {
        a.residue = a.residue.square();
    }

    /// Compute `a^{-1}` if `a` is not zero.
    #[must_use]
    fn inverse(a: &Fp<Self, LIMBS>) -> Option<Fp<Self, LIMBS>> {
        let (residue, choice) = a.residue.invert();
        let is_inverse: bool = choice.into();

        is_inverse.then_some(Fp { residue })
    }

    /// Construct a field element from an integer.
    ///
    /// By the end element will be converted to a montgomery form and reduced.
    #[must_use]
    fn from_bigint(r: Uint<LIMBS>) -> Fp<Self, LIMBS> {
        Fp::new(r)
    }

    /// Convert a field element to an integer less than [`Self::MODULUS`].
    #[must_use]
    fn into_bigint(a: Fp<Self, LIMBS>) -> Uint<LIMBS> {
        a.residue.retrieve()
    }
}

/// Represents an element of the prime field `F_p`, where `p == P::MODULUS`.
///
/// This type can represent elements in any field of size at most N * 64 bits
/// for 64-bit systems and N * 32 bits for 32-bit systems.
#[derive(Educe)]
#[educe(Default, Clone, Copy, PartialEq, Eq)]
pub struct Fp<P: FpParams<LIMBS>, const LIMBS: usize> {
    /// Contains the element in Montgomery form for efficient multiplication.
    /// To convert an element to a [`BigInt`], use [`FpParams::into_bigint`]
    /// or `into`.
    residue: Residue<ResidueParam<P, LIMBS>, LIMBS>,
}

/// Declare [`Fp`] types for different bit sizes.
macro_rules! declare_fp {
    ($fp:ident, $limbs:ident, $bits:expr) => {
        #[doc = "Finite field with max"]
        #[doc = stringify!($bits)]
        #[doc = "bits size element."]
        pub type $fp<P> = crate::field::fp::Fp<
            P,
            { usize::div_ceil($bits, ::crypto_bigint::Word::BITS as usize) },
        >;

        #[doc = "Number of limbs in the field with"]
        #[doc = stringify!($bits)]
        #[doc = "bits size element."]
        pub const $limbs: usize =
            usize::div_ceil($bits, ::crypto_bigint::Word::BITS as usize);
    };
}

declare_fp!(Fp64, LIMBS_64, 64);
declare_fp!(Fp128, LIMBS_128, 128);
declare_fp!(Fp192, LIMBS_192, 192);
declare_fp!(Fp256, LIMBS_256, 256);
declare_fp!(Fp320, LIMBS_320, 320);
declare_fp!(Fp384, LIMBS_384, 384);
declare_fp!(Fp448, LIMBS_448, 448);
declare_fp!(Fp512, LIMBS_512, 512);
declare_fp!(Fp576, LIMBS_576, 576);
declare_fp!(Fp640, LIMBS_640, 640);
declare_fp!(Fp704, LIMBS_704, 704);
declare_fp!(Fp768, LIMBS_768, 768);
declare_fp!(Fp832, LIMBS_832, 832);

#[derive(Educe)]
#[educe(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResidueParam<P: FpParams<LIMBS>, const LIMBS: usize>(PhantomData<P>);

impl<P: FpParams<LIMBS>, const LIMBS: usize> ResidueParams<LIMBS>
    for ResidueParam<P, LIMBS>
{
    const LIMBS: usize = LIMBS;
    const MODULUS: Uint<LIMBS> = {
        let modulus = P::MODULUS;
        // Uint represents integer in low-endian form.
        assert!(modulus.as_limbs()[0].0 & 1 == 1, "modulus must be odd");
        modulus
    };
    const MOD_NEG_INV: Limb = Limb(Word::MIN.wrapping_sub(
        P::MODULUS.inv_mod2k_vartime(Word::BITS as usize).as_limbs()[0].0,
    ));
    const R: Uint<LIMBS> =
        Uint::MAX.const_rem(&P::MODULUS).0.wrapping_add(&Uint::ONE);
    const R2: Uint<LIMBS> =
        Uint::const_rem_wide(Self::R.square_wide(), &P::MODULUS).0;
    const R3: Uint<LIMBS> = montgomery_reduction(
        &Self::R2.square_wide(),
        &P::MODULUS,
        Self::MOD_NEG_INV,
    );
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> Fp<P, LIMBS> {
    /// A multiplicative generator of the field.
    /// [`Self::GENERATOR`] is an element having multiplicative order
    /// `MODULUS - 1`.
    ///
    /// Every element of the field should be represented as `GENERATOR^i`
    pub const GENERATOR: Fp<P, LIMBS> = P::GENERATOR;
    /// Multiplicative identity of the field, i.e., the element `e`
    /// such that, for all elements `f` of the field, `e * f = f`.
    pub const ONE: Fp<P, LIMBS> = Fp::new_unchecked(Self::R);
    /// Let `M` be the power of 2^64 nearest to [`Self::MODULUS_BITS`]. Then
    /// `R = M % MODULUS`.
    const R: Uint<LIMBS> = ResidueParam::<P, LIMBS>::R;
    /// `R2 = R^2 % MODULUS`
    #[allow(dead_code)]
    const R2: Uint<LIMBS> = ResidueParam::<P, LIMBS>::R2;
    /// Additive identity of the field, i.e., the element `e`
    /// such that, for all elements `f` of the field, `e + f = f`.
    pub const ZERO: Fp<P, LIMBS> = Fp::new_unchecked(Uint::ZERO);

    /// Construct a new field element from [`Uint`] and convert it in
    /// Montgomery form.
    #[inline]
    #[must_use]
    pub const fn new(element: Uint<LIMBS>) -> Self {
        Fp { residue: Residue::<ResidueParam<P, LIMBS>, LIMBS>::new(&element) }
    }

    /// Construct a new field element from [`Uint`].
    ///
    /// Unlike [`Self::new`], this method does not perform Montgomery reduction.
    /// This method should be used only when constructing an element from an
    /// integer that has already been put in Montgomery form.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(element: Uint<LIMBS>) -> Self {
        Fp {
            residue: Residue::<ResidueParam<P, LIMBS>, LIMBS>::from_montgomery(
                element,
            ),
        }
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> Hash for Fp<P, LIMBS> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.residue.as_montgomery().hash(state);
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> Debug for Fp<P, LIMBS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.into_bigint(), f)
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> Zero for Fp<P, LIMBS> {
    #[inline]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> One for Fp<P, LIMBS> {
    #[inline]
    fn one() -> Self {
        Self::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> AdditiveGroup for Fp<P, LIMBS> {
    type Scalar = Self;

    const ZERO: Self = Self::ZERO;

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

impl<P: FpParams<LIMBS>, const LIMBS: usize> Field for Fp<P, LIMBS> {
    const ONE: Self = Fp::new_unchecked(Self::R);

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
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> PrimeField for Fp<P, LIMBS> {
    type BigInt = Uint<LIMBS>;

    const MODULUS: Self::BigInt = P::MODULUS;
    const MODULUS_BIT_SIZE: usize = P::MODULUS.bits();

    #[inline]
    fn from_bigint(repr: Self::BigInt) -> Self {
        P::from_bigint(repr)
    }

    fn into_bigint(self) -> Uint<LIMBS> {
        P::into_bigint(self)
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> Ord for Fp<P, LIMBS> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_bigint().cmp(&other.into_bigint())
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> PartialOrd for Fp<P, LIMBS> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Auto implements conversion from unsigned integer of type `$int` to [`Fp`].
macro_rules! impl_fp_from_unsigned_int {
    ($int:ty) => {
        impl<P: FpParams<LIMBS>, const LIMBS: usize> From<$int>
            for Fp<P, LIMBS>
        {
            fn from(other: $int) -> Self {
                Fp::from_bigint(Uint::from(other))
            }
        }
    };
}

/// Auto implements conversion from signed integer of type `$int` to [`Fp`].
macro_rules! impl_fp_from_signed_int {
    ($int:ty) => {
        impl<P: FpParams<LIMBS>, const LIMBS: usize> From<$int>
            for Fp<P, LIMBS>
        {
            fn from(other: $int) -> Self {
                let abs = other.unsigned_abs().into();
                if other.is_positive() {
                    abs
                } else {
                    -abs
                }
            }
        }
    };
}

impl_fp_from_unsigned_int!(u128);
impl_fp_from_unsigned_int!(u64);
impl_fp_from_unsigned_int!(u32);
impl_fp_from_unsigned_int!(u16);
impl_fp_from_unsigned_int!(u8);

impl_fp_from_signed_int!(i128);
impl_fp_from_signed_int!(i64);
impl_fp_from_signed_int!(i32);
impl_fp_from_signed_int!(i16);
impl_fp_from_signed_int!(i8);

impl<P: FpParams<LIMBS>, const LIMBS: usize> From<bool> for Fp<P, LIMBS> {
    fn from(other: bool) -> Self {
        u8::from(other).into()
    }
}

/// Auto implements conversion from [`Fp`] to integer of type `$int`.
///
/// Conversion is available only for a single limb field elements,
/// i.e. `LIMBS = 1`.
macro_rules! impl_int_from_fp {
    ($int:ty) => {
        impl<P: FpParams<1>> From<Fp<P, 1>> for $int {
            fn from(other: Fp<P, 1>) -> Self {
                let uint = other.into_bigint();
                let words = uint.as_words();
                <$int>::try_from(words[0]).unwrap_or_else(|_| {
                    panic!("should convert to {}", stringify!($int))
                })
            }
        }
    };
}

impl_int_from_fp!(u128);
impl_int_from_fp!(u64);
impl_int_from_fp!(u32);
impl_int_from_fp!(u16);
impl_int_from_fp!(u8);
impl_int_from_fp!(i128);
impl_int_from_fp!(i64);
impl_int_from_fp!(i32);
impl_int_from_fp!(i16);
impl_int_from_fp!(i8);

#[cfg(test)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> crypto_bigint::Random
    for Fp<P, LIMBS>
{
    #[inline]
    fn random(rng: &mut impl crypto_bigint::rand_core::CryptoRngCore) -> Self {
        Fp { residue: Residue::<ResidueParam<P, LIMBS>, LIMBS>::random(rng) }
    }
}

/// Outputs a string containing the value of `self`,
/// represented as a decimal without leading zeroes.
impl<P: FpParams<LIMBS>, const LIMBS: usize> Display for Fp<P, LIMBS> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = self.into_bigint().to_string();
        write!(f, "{str}")
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Neg for Fp<P, LIMBS> {
    type Output = Self;

    #[inline]
    fn neg(mut self) -> Self {
        P::neg_in_place(&mut self);
        self
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Add<&Fp<P, LIMBS>>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(other);
        self
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Sub<&Fp<P, LIMBS>>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(other);
        self
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Mul<&Fp<P, LIMBS>>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(other);
        self
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Div<&Fp<P, LIMBS>>
    for Fp<P, LIMBS>
{
    type Output = Self;

    /// Returns `self * other.inverse()` if `other.inverse()` is `Some`, and
    /// panics otherwise.
    #[inline]
    fn div(mut self, other: &Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&other.inverse().unwrap());
        self
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Add<&Fp<P, LIMBS>>
    for &Fp<P, LIMBS>
{
    type Output = Fp<P, LIMBS>;

    #[inline]
    fn add(self, other: &Fp<P, LIMBS>) -> Fp<P, LIMBS> {
        use core::ops::AddAssign;
        let mut result = *self;
        result.add_assign(other);
        result
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Sub<&Fp<P, LIMBS>>
    for &Fp<P, LIMBS>
{
    type Output = Fp<P, LIMBS>;

    #[inline]
    fn sub(self, other: &Fp<P, LIMBS>) -> Fp<P, LIMBS> {
        use core::ops::SubAssign;
        let mut result = *self;
        result.sub_assign(other);
        result
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Mul<&Fp<P, LIMBS>>
    for &Fp<P, LIMBS>
{
    type Output = Fp<P, LIMBS>;

    #[inline]
    fn mul(self, other: &Fp<P, LIMBS>) -> Fp<P, LIMBS> {
        use core::ops::MulAssign;
        let mut result = *self;
        result.mul_assign(other);
        result
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Div<&Fp<P, LIMBS>>
    for &Fp<P, LIMBS>
{
    type Output = Fp<P, LIMBS>;

    #[inline]
    fn div(self, other: &Fp<P, LIMBS>) -> Fp<P, LIMBS> {
        use core::ops::DivAssign;
        let mut result = *self;
        result.div_assign(other);
        result
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::AddAssign<&Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        P::add_assign(self, other);
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::SubAssign<&Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn sub_assign(&mut self, other: &Self) {
        P::sub_assign(self, other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Add<Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Add<&mut Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &mut Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Sub<Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Sub<&mut Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &mut Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::iter::Sum<Self>
    for Fp<P, LIMBS>
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<LIMBS>, const LIMBS: usize> core::iter::Sum<&'a Self>
    for Fp<P, LIMBS>
{
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::AddAssign<Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::SubAssign<Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::AddAssign<&mut Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn add_assign(&mut self, other: &mut Self) {
        self.add_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::SubAssign<&mut Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn sub_assign(&mut self, other: &mut Self) {
        self.sub_assign(&*other);
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::MulAssign<&Self>
    for Fp<P, LIMBS>
{
    fn mul_assign(&mut self, other: &Self) {
        P::mul_assign(self, other);
    }
}

/// Computes `self *= other.inverse()` if `other.inverse()` is `Some`, and
/// panics otherwise.
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::DivAssign<&Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn div_assign(&mut self, other: &Self) {
        use core::ops::MulAssign;
        self.mul_assign(&other.inverse().expect("should not divide by zero"));
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Mul<Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn mul(mut self, other: Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Div<Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn div(mut self, other: Self) -> Self {
        use core::ops::DivAssign;
        self.div_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Mul<&mut Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &mut Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::Div<&mut Self>
    for Fp<P, LIMBS>
{
    type Output = Self;

    #[inline]
    fn div(mut self, other: &mut Self) -> Self {
        use core::ops::DivAssign;
        self.div_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::iter::Product<Self>
    for Fp<P, LIMBS>
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<LIMBS>, const LIMBS: usize> core::iter::Product<&'a Self>
    for Fp<P, LIMBS>
{
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::MulAssign<Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        self.mul_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::DivAssign<&mut Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn div_assign(&mut self, other: &mut Self) {
        self.div_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::MulAssign<&mut Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn mul_assign(&mut self, other: &mut Self) {
        self.mul_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<LIMBS>, const LIMBS: usize> core::ops::DivAssign<Self>
    for Fp<P, LIMBS>
{
    #[inline]
    fn div_assign(&mut self, other: Self) {
        self.div_assign(&other);
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> zeroize::Zeroize for Fp<P, LIMBS> {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        self.residue.zeroize();
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> From<Fp<P, LIMBS>>
    for Uint<LIMBS>
{
    #[inline]
    fn from(fp: Fp<P, LIMBS>) -> Self {
        fp.into_bigint()
    }
}

impl<P: FpParams<LIMBS>, const LIMBS: usize> From<Uint<LIMBS>>
    for Fp<P, LIMBS>
{
    /// Converts `Self::BigInteger` into `Self`
    #[inline]
    fn from(int: Uint<LIMBS>) -> Self {
        Self::from_bigint(int)
    }
}

/// This macro converts a string base-10 number to a field element.
#[macro_export]
macro_rules! fp_from_num {
    ($num:literal) => {
        $crate::field::fp::Fp::new($crate::bigint::from_str_radix($num, 10))
    };
}

/// This macro converts a string hex number to a field element.
#[macro_export]
macro_rules! fp_from_hex {
    ($num:literal) => {{
        $crate::field::fp::Fp::new(
            $crate::bigint::crypto_bigint::Uint::from_be_hex($num),
        )
    }};
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::{
        bigint::crypto_bigint::U64,
        field::{
            fp::{Fp64, FpParams, LIMBS_64},
            group::AdditiveGroup,
            prime::PrimeField,
        },
        fp_from_num, from_num,
    };

    type Field64 = Fp64<Fp64Param>;
    struct Fp64Param;
    impl FpParams<LIMBS_64> for Fp64Param {
        const GENERATOR: Fp64<Fp64Param> = fp_from_num!("3");
        const MODULUS: U64 = from_num!("1000003"); // Prime number
    }

    const MODULUS: i128 = 1000003; // Prime number

    proptest! {
        #[test]
        fn add(a: i64, b: i64) {
            let res = Field64::from(a) + Field64::from(b);
            let res: i128 = res.into();
            let a = a as i128;
            let b = b as i128;
            prop_assert_eq!(res, (a + b).rem_euclid(MODULUS));
        }

        #[test]
        fn double(a: i64) {
            let res = Field64::from(a).double();
            let res: i128 = res.into();
            let a = a as i128;
            prop_assert_eq!(res, (a + a).rem_euclid(MODULUS));
        }

        #[test]
        fn sub(a: i64, b: i64) {
            let res = Field64::from(a) - Field64::from(b);
            let res: i128 = res.into();
            let a = a as i128;
            let b = b as i128;
            prop_assert_eq!(res, (a - b).rem_euclid(MODULUS));
        }

        #[test]
        fn mul(a: i64, b: i64) {
            let res = Field64::from(a) * Field64::from(b);
            let res: i128 = res.into();
            let a = a as i128;
            let b = b as i128;
            prop_assert_eq!(res, (a * b).rem_euclid(MODULUS));
        }

        #[test]
        fn square(a: i64) {
            let res = Field64::from(a).square();
            let res: i128 = res.into();
            let a = a as i128;
            prop_assert_eq!(res, (a * a).rem_euclid(MODULUS));
        }

        #[test]
        fn div(a: i64, b: i64) {
            // Skip if `b` is zero.
            if (b as i128) % MODULUS == 0 {
                return Ok(());
            }

            let res = Field64::from(a) / Field64::from(b);
            let res: i128 = res.into();
            let a = a as i128;
            let b = b as i128;
            // a / b = res mod M => res * b = a mod M
            prop_assert_eq!((res * b).rem_euclid(MODULUS), a.rem_euclid(MODULUS));
        }

        #[test]
        fn pow(a: i64, b in 0_u32..1000) {
            /// Compute a^b in an expensive and iterative way.
            fn dumb_pow(a: i128, b: i128) -> i128 {
                (0..b).fold(1, |acc, _| (acc * a).rem_euclid(MODULUS))
            }

            let res = Field64::from(a).pow(b);
            let res: i128 = res.into();
            let a = a as i128;
            let b = b as i128;
            prop_assert_eq!(res, dumb_pow(a, b));
        }

        #[test]
        fn neg(a: i64) {
            let res = -Field64::from(a);
            let res: i128 = res.into();
            let a = a as i128;
            prop_assert_eq!(res, (-a).rem_euclid(MODULUS));
        }

        #[test]
        fn one(a: i64) {
            let res = Field64::one();
            let res: i128 = res.into();
            prop_assert_eq!(res, 1);

            let res = Field64::one() * Field64::from(a);
            let res: i128 = res.into();
            let a: i128 = a.into();
            prop_assert_eq!(res, a.rem_euclid(MODULUS));
        }

        #[test]
        fn zero(a: i64) {
            let res = Field64::zero();
            let res: i128 = res.into();
            prop_assert_eq!(res, 0);

            let res = Field64::zero() + Field64::from(a);
            let res: i128 = res.into();
            let a: i128 = a.into();
            prop_assert_eq!(res, a.rem_euclid(MODULUS));
        }
    }
}
