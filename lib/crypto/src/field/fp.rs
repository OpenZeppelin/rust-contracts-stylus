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
pub trait FpParams<const N: usize>: Send + Sync + 'static + Sized {
    /// The modulus of the field.
    const MODULUS: Uint<N>;

    /// A multiplicative generator of the field.
    /// [`Self::GENERATOR`] is an element having multiplicative order
    /// `MODULUS - 1`.
    const GENERATOR: Fp<Self, N>;

    /// Additive identity of the field, i.e., the element `e`
    /// such that, for all elements `f` of the field, `e + f = f`.
    const ZERO: Fp<Self, N> = Fp::new_unchecked(Uint::ZERO);

    /// Multiplicative identity of the field, i.e., the element `e`
    /// such that, for all elements `f` of the field, `e * f = f`.
    const ONE: Fp<Self, N> = Fp::new_unchecked(Self::R);

    /// Let `M` be the power of 2^64 nearest to [`Self::MODULUS_BITS`]. Then
    /// `R = M % MODULUS`.
    const R: Uint<N> = ResidueParam::<Self, N>::R;

    /// `R2 = R^2 % MODULUS`
    const R2: Uint<N> = ResidueParam::<Self, N>::R2;

    /// `INV = -MODULUS^{-1} mod 2^64`
    const INV: Word = ResidueParam::<Self, N>::MOD_NEG_INV.0;

    /// Set `a += b`.
    fn add_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        a.residue += b.residue;
    }

    /// Set `a -= b`.
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        a.residue -= b.residue;
    }

    /// Set `a = a + a`.
    fn double_in_place(a: &mut Fp<Self, N>) {
        a.residue = a.residue + a.residue;
    }

    /// Set `a = -a`;
    fn neg_in_place(a: &mut Fp<Self, N>) {
        a.residue = a.residue.neg();
    }

    /// Set `a *= b`.
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        a.residue *= b.residue;
    }

    /// Set `a *= a`.
    fn square_in_place(a: &mut Fp<Self, N>) {
        a.residue = a.residue.square();
    }

    /// Compute `a^{-1}` if `a` is not zero.
    #[must_use]
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        let (residue, choice) = a.residue.invert();
        let is_inverse: bool = choice.into();

        is_inverse.then_some(Fp { residue })
    }

    /// Construct a field element from an integer.
    ///
    /// By the end element will be converted to a montgomery form and reduced.
    #[must_use]
    fn from_bigint(r: Uint<N>) -> Fp<Self, N> {
        Fp::new(r)
    }

    /// Convert a field element to an integer less than [`Self::MODULUS`].
    #[must_use]
    fn into_bigint(a: Fp<Self, N>) -> Uint<N> {
        a.residue.retrieve()
    }
}

/// Represents an element of the prime field `F_p`, where `p == P::MODULUS`.
///
/// This type can represent elements in any field of size at most N * 64 bits
/// for 64-bit systems and N * 32 bits for 32-bit systems.
#[derive(Educe)]
#[educe(Default, Clone, Copy, PartialEq, Eq)]
pub struct Fp<P: FpParams<N>, const N: usize> {
    /// Contains the element in Montgomery form for efficient multiplication.
    /// To convert an element to a [`BigInt`], use [`FpParams::into_bigint`]
    /// or `into`.
    residue: Residue<ResidueParam<P, N>, N>,
}

impl<P: FpParams<N>, const N: usize> Hash for Fp<P, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.residue.as_montgomery().hash(state);
    }
}

/// Declare [`Fp`] type for different bit sizes.
macro_rules! declare_fp {
    ($name:ident, $bits:expr) => {
        #[cfg(target_pointer_width = "64")]
        #[doc = "Finite field with max"]
        #[doc = stringify!($bits)]
        #[doc = "bits size per element."]
        pub type $name<P> =
            crate::field::fp::Fp<P, { usize::div_ceil($bits, 64) }>;

        #[cfg(target_pointer_width = "32")]
        #[doc = "Finite field with max"]
        #[doc = stringify!($bits)]
        #[doc = "bits size per element."]
        pub type $name<P> =
            crate::field::fp::Fp<P, { usize::div_ceil($bits, 32) }>;
    };
}

declare_fp!(Fp64, 64);
declare_fp!(Fp128, 128);
declare_fp!(Fp192, 192);
declare_fp!(Fp256, 256);
declare_fp!(Fp320, 320);
declare_fp!(Fp384, 384);
declare_fp!(Fp448, 448);
declare_fp!(Fp512, 512);
declare_fp!(Fp576, 576);
declare_fp!(Fp640, 640);
declare_fp!(Fp704, 704);
declare_fp!(Fp768, 768);
declare_fp!(Fp832, 832);

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

impl<P: FpParams<N>, const N: usize> Fp<P, N> {
    #[doc(hidden)]
    pub const INV: Word = P::INV;
    #[doc(hidden)]
    pub const R: Uint<N> = P::R;
    #[doc(hidden)]
    pub const R2: Uint<N> = P::R2;

    /// Construct a new field element from [`Uint`] and convert it in
    /// Montgomery form.
    #[inline]
    #[must_use]
    pub const fn new(element: Uint<N>) -> Self {
        Fp { residue: Residue::<ResidueParam<P, N>, N>::new(&element) }
    }

    /// Construct a new field element from [`Uint`].
    ///
    /// Unlike [`Self::new`], this method does not perform Montgomery reduction.
    /// This method should be used only when constructing an element from an
    /// integer that has already been put in Montgomery form.
    #[inline]
    #[must_use]
    pub const fn new_unchecked(element: Uint<N>) -> Self {
        Fp {
            residue: Residue::<ResidueParam<P, N>, N>::from_montgomery(element),
        }
    }
}

impl<P: FpParams<N>, const N: usize> Debug for Fp<P, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.into_bigint(), f)
    }
}

impl<P: FpParams<N>, const N: usize> Zero for Fp<P, N> {
    #[inline]
    fn zero() -> Self {
        P::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == P::ZERO
    }
}

impl<P: FpParams<N>, const N: usize> One for Fp<P, N> {
    #[inline]
    fn one() -> Self {
        P::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        *self == P::ONE
    }
}

impl<P: FpParams<N>, const N: usize> AdditiveGroup for Fp<P, N> {
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

impl<P: FpParams<N>, const N: usize> Field for Fp<P, N> {
    const ONE: Self = P::ONE;

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

impl<P: FpParams<N>, const N: usize> PrimeField for Fp<P, N> {
    type BigInt = Uint<N>;

    const MODULUS: Self::BigInt = P::MODULUS;
    const MODULUS_BIT_SIZE: usize = P::MODULUS.bits();

    #[inline]
    fn from_bigint(repr: Self::BigInt) -> Self {
        P::from_bigint(repr)
    }

    fn into_bigint(self) -> Uint<N> {
        P::into_bigint(self)
    }
}

impl<P: FpParams<N>, const N: usize> Ord for Fp<P, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_bigint().cmp(&other.into_bigint())
    }
}

impl<P: FpParams<N>, const N: usize> PartialOrd for Fp<P, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<u128> for Fp<P, N> {
    fn from(other: u128) -> Self {
        Fp::from_bigint(Uint::from_u128(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<i128> for Fp<P, N> {
    fn from(other: i128) -> Self {
        other.unsigned_abs().into()
    }
}

impl<P: FpParams<N>, const N: usize> From<bool> for Fp<P, N> {
    fn from(other: bool) -> Self {
        u8::from(other).into()
    }
}

impl<P: FpParams<N>, const N: usize> From<u64> for Fp<P, N> {
    fn from(other: u64) -> Self {
        Fp::from_bigint(Uint::from_u64(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<i64> for Fp<P, N> {
    fn from(other: i64) -> Self {
        other.unsigned_abs().into()
    }
}

impl<P: FpParams<N>, const N: usize> From<u32> for Fp<P, N> {
    fn from(other: u32) -> Self {
        Fp::from_bigint(Uint::from_u32(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<i32> for Fp<P, N> {
    fn from(other: i32) -> Self {
        other.unsigned_abs().into()
    }
}

impl<P: FpParams<N>, const N: usize> From<u16> for Fp<P, N> {
    fn from(other: u16) -> Self {
        Fp::from_bigint(Uint::from_u16(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<i16> for Fp<P, N> {
    fn from(other: i16) -> Self {
        other.unsigned_abs().into()
    }
}

impl<P: FpParams<N>, const N: usize> From<u8> for Fp<P, N> {
    fn from(other: u8) -> Self {
        Fp::from_bigint(Uint::from_u8(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<i8> for Fp<P, N> {
    fn from(other: i8) -> Self {
        other.unsigned_abs().into()
    }
}

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
impl<P: FpParams<N>, const N: usize> Display for Fp<P, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = self.into_bigint().to_string();
        write!(f, "{str}")
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::Neg for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn neg(mut self) -> Self {
        P::neg_in_place(&mut self);
        self
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::Add<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(other);
        self
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::Sub<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(other);
        self
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::Mul<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(other);
        self
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::Div<&Fp<P, N>> for Fp<P, N> {
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

impl<'a, 'b, P: FpParams<N>, const N: usize> core::ops::Add<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn add(self, other: &'b Fp<P, N>) -> Fp<P, N> {
        use core::ops::AddAssign;
        let mut result = *self;
        result.add_assign(other);
        result
    }
}

impl<'a, 'b, P: FpParams<N>, const N: usize> core::ops::Sub<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn sub(self, other: &Fp<P, N>) -> Fp<P, N> {
        use core::ops::SubAssign;
        let mut result = *self;
        result.sub_assign(other);
        result
    }
}

impl<'a, 'b, P: FpParams<N>, const N: usize> core::ops::Mul<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn mul(self, other: &Fp<P, N>) -> Fp<P, N> {
        use core::ops::MulAssign;
        let mut result = *self;
        result.mul_assign(other);
        result
    }
}

impl<'a, 'b, P: FpParams<N>, const N: usize> core::ops::Div<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn div(self, other: &Fp<P, N>) -> Fp<P, N> {
        use core::ops::DivAssign;
        let mut result = *self;
        result.div_assign(other);
        result
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::AddAssign<&Self> for Fp<P, N> {
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        P::add_assign(self, other);
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::SubAssign<&Self> for Fp<P, N> {
    #[inline]
    fn sub_assign(&mut self, other: &Self) {
        P::sub_assign(self, other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Add<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Add<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &mut Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Sub<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Sub<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &mut Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::iter::Sum<Self> for Fp<P, N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::iter::Sum<&'a Self>
    for Fp<P, N>
{
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::AddAssign<Self> for Fp<P, N> {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::SubAssign<Self> for Fp<P, N> {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::AddAssign<&mut Self>
    for Fp<P, N>
{
    #[inline]
    fn add_assign(&mut self, other: &mut Self) {
        self.add_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::SubAssign<&mut Self>
    for Fp<P, N>
{
    #[inline]
    fn sub_assign(&mut self, other: &mut Self) {
        self.sub_assign(&*other);
    }
}

impl<P: FpParams<N>, const N: usize> core::ops::MulAssign<&Self> for Fp<P, N> {
    fn mul_assign(&mut self, other: &Self) {
        P::mul_assign(self, other);
    }
}

/// Computes `self *= other.inverse()` if `other.inverse()` is `Some`, and
/// panics otherwise.
impl<P: FpParams<N>, const N: usize> core::ops::DivAssign<&Self> for Fp<P, N> {
    #[inline]
    fn div_assign(&mut self, other: &Self) {
        use core::ops::MulAssign;
        self.mul_assign(&other.inverse().unwrap());
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Mul<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Div<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn div(mut self, other: Self) -> Self {
        use core::ops::DivAssign;
        self.div_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Mul<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &mut Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Div<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn div(mut self, other: &mut Self) -> Self {
        use core::ops::DivAssign;
        self.div_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::iter::Product<Self> for Fp<P, N> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::iter::Product<&'a Self>
    for Fp<P, N>
{
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::MulAssign<Self> for Fp<P, N> {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        self.mul_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::DivAssign<&mut Self>
    for Fp<P, N>
{
    #[inline]
    fn div_assign(&mut self, other: &mut Self) {
        self.div_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::MulAssign<&mut Self>
    for Fp<P, N>
{
    #[inline]
    fn mul_assign(&mut self, other: &mut Self) {
        self.mul_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::DivAssign<Self> for Fp<P, N> {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        self.div_assign(&other);
    }
}

impl<P: FpParams<N>, const N: usize> zeroize::Zeroize for Fp<P, N> {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        self.residue.zeroize();
    }
}

impl<P: FpParams<N>, const N: usize> From<Fp<P, N>> for Uint<N> {
    #[inline]
    fn from(fp: Fp<P, N>) -> Self {
        fp.into_bigint()
    }
}

impl<P: FpParams<N>, const N: usize> From<Uint<N>> for Fp<P, N> {
    /// Converts `Self::BigInteger` into `Self`
    #[inline]
    fn from(int: Uint<N>) -> Self {
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
