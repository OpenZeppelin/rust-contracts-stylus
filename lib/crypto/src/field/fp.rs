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
    /// `Self::GENERATOR` is an element having multiplicative order
    /// `Self::MODULUS - 1`.
    const GENERATOR: Fp<Self, N>;

    /// Additive identity of the field, i.e. the element `e`
    /// such that, for all elements `f` of the field, `e + f = f`.
    const ZERO: Fp<Self, N> = Fp::new_unchecked(Uint::ZERO);

    /// Multiplicative identity of the field, i.e. the element `e`
    /// such that, for all elements `f` of the field, `e * f = f`.
    const ONE: Fp<Self, N> = Fp::new_unchecked(Self::R);

    /// Let `M` be the power of 2^64 nearest to `Self::MODULUS_BITS`. Then
    /// `R = M % Self::MODULUS`.
    const R: Uint<N> = ResidueParam::<Self, N>::R;

    /// R2 = R^2 % Self::MODULUS
    const R2: Uint<N> = ResidueParam::<Self, N>::R2;

    /// INV = -MODULUS^{-1} mod 2^64
    const INV: u64 = ResidueParam::<Self, N>::MOD_NEG_INV.0;

    /// Set a += b.
    fn add_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        a.residue += b.residue;
    }

    /// Set a -= b.
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        a.residue -= b.residue;
    }

    /// Set a = a + a.
    fn double_in_place(a: &mut Fp<Self, N>) {
        a.residue = a.residue + a.residue;
    }

    /// Set a = -a;
    fn neg_in_place(a: &mut Fp<Self, N>) {
        a.residue = a.residue.neg();
    }

    /// Set a *= b.
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        a.residue *= b.residue;
    }

    /// Set a *= a.
    fn square_in_place(a: &mut Fp<Self, N>) {
        a.residue = a.residue.square();
    }

    /// Compute a^{-1} if `a` is not zero.
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        let (residue, choice) = a.residue.invert();
        let is_inverse: bool = choice.into();

        is_inverse.then_some(Fp { residue })
    }

    // TODO#q: do we need to return Option<_> here?
    /// Construct a field element from an integer from `0` to `Self::MODULUS -
    /// 1`.
    /// Returns `None` if the integer is outside this range.
    fn from_bigint(r: Uint<N>) -> Option<Fp<Self, N>> {
        if r >= Self::MODULUS {
            None
        } else {
            Some(Fp::new(r))
        }
    }

    /// Convert a field element to an integer from `0` to `Self::MODULUS - 1`.
    fn into_bigint(a: Fp<Self, N>) -> Uint<N> {
        a.residue.retrieve()
    }
}

/// Represents an element of the prime field F_p, where `p == P::MODULUS`.
/// This type can represent elements in any field of size at most N * 64 bits.
#[derive(Educe)]
#[educe(Default, Clone, Copy, PartialEq, Eq)]
pub struct Fp<P: FpParams<N>, const N: usize> {
    /// Contains the element in Montgomery form for efficient multiplication.
    /// To convert an element to a [`BigInt`](struct@BigInt), use `into_bigint`
    /// or `into`.
    residue: Residue<ResidueParam<P, N>, N>,
}

impl<P: FpParams<N>, const N: usize> Hash for Fp<P, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.residue.as_montgomery().hash(state);
    }
}

// TODO#q: add doc comments with macro
pub type Fp64<P> = Fp<P, 1>;
pub type Fp128<P> = Fp<P, 2>;
pub type Fp192<P> = Fp<P, 3>;
pub type Fp256<P> = Fp<P, 4>;
pub type Fp320<P> = Fp<P, 5>;
pub type Fp384<P> = Fp<P, 6>;
pub type Fp448<P> = Fp<P, 7>;
pub type Fp512<P> = Fp<P, 8>;
pub type Fp576<P> = Fp<P, 9>;
pub type Fp640<P> = Fp<P, 10>;
pub type Fp704<P> = Fp<P, 11>;
pub type Fp768<P> = Fp<P, 12>;
pub type Fp832<P> = Fp<P, 13>;

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
    pub const INV: u64 = P::INV;
    #[doc(hidden)]
    pub const R: Uint<N> = P::R;
    #[doc(hidden)]
    pub const R2: Uint<N> = P::R2;

    /// Construct a new field element from [`Uint`] and convert it in
    /// Montgomery form.
    #[inline]
    pub const fn new(element: Uint<N>) -> Self {
        Fp { residue: Residue::<ResidueParam<P, N>, N>::new(&element) }
    }

    /// Construct a new field element from [`Uint`].
    ///
    /// Unlike [`Self::new`], this method does not perform Montgomery reduction.
    /// This method should be used only when constructing an element from an
    /// integer that has already been put in Montgomery form.
    #[inline]
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

    fn extension_degree() -> u64 {
        1
    }

    #[inline]
    fn characteristic() -> &'static [u64] {
        P::MODULUS.as_words()
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
}

impl<P: FpParams<N>, const N: usize> PrimeField for Fp<P, N> {
    type BigInt = Uint<N>;

    const MODULUS: Self::BigInt = P::MODULUS;
    const MODULUS_BIT_SIZE: usize = P::MODULUS.bits();

    #[inline]
    fn from_bigint(r: Uint<N>) -> Option<Self> {
        P::from_bigint(r)
    }

    fn into_bigint(self) -> Uint<N> {
        P::into_bigint(self)
    }
}

// TODO#q: rephrase comment
/// Note that this implementation of `Ord` compares field elements viewing
/// them as integers in the range 0, 1, ..., P::MODULUS - 1. However, other
/// implementations of `PrimeField` might choose a different ordering, and
/// as such, users should use this `Ord` for applications where
/// any ordering suffices (like in a BTreeMap), and not in applications
/// where a particular ordering is required.
impl<P: FpParams<N>, const N: usize> Ord for Fp<P, N> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.into_bigint().cmp(&other.into_bigint())
    }
}

// TODO#q: rephrase comment
/// Note that this implementation of `PartialOrd` compares field elements
/// viewing them as integers in the range 0, 1, ..., `P::MODULUS` - 1. However,
/// other implementations of `PrimeField` might choose a different ordering, and
/// as such, users should use this `PartialOrd` for applications where
/// any ordering suffices (like in a BTreeMap), and not in applications
/// where a particular ordering is required.
impl<P: FpParams<N>, const N: usize> PartialOrd for Fp<P, N> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: FpParams<N>, const N: usize> From<u128> for Fp<P, N> {
    fn from(other: u128) -> Self {
        Fp::from_bigint(Uint::from_u128(other))
            .expect("should be less than modulus")
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
            .expect("should be less than modulus")
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
            .expect("should be less than modulus")
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
            .expect("should be less than modulus")
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
            .expect("should be less than modulus")
    }
}

impl<P: FpParams<N>, const N: usize> From<i8> for Fp<P, N> {
    fn from(other: i8) -> Self {
        other.unsigned_abs().into()
    }
}

// TODO#q: add rand Distribution trait
// impl<P: FpConfig<N>, const N: usize>
//     ark_std::rand::distributions::Distribution<Fp<P, N>>
//     for ark_std::rand::distributions::Standard
// {
//     #[inline]
//     fn sample<R: ark_std::rand::Rng + ?Sized>(&self, rng: &mut R) -> Fp<P, N>
// {         loop {
//             let mut tmp = Fp(
//                 rng.sample(ark_std::rand::distributions::Standard),
//                 PhantomData,
//             );
//             let shave_bits = Fp::<P, N>::num_bits_to_shave();
//             // Mask away the unused bits at the beginning.
//             assert!(shave_bits <= 64);
//             let mask =
//                 if shave_bits == 64 { 0 } else { u64::MAX >> shave_bits };
//
//             if let Some(val) = tmp.0 .0.last_mut() {
//                 *val &= mask
//             }
//
//             if !tmp.is_geq_modulus() {
//                 return tmp;
//             }
//         }
//     }
// }

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
    #[must_use]
    fn neg(mut self) -> Self {
        P::neg_in_place(&mut self);
        self
    }
}

impl<'a, P: FpParams<N>, const N: usize> core::ops::Add<&'a Fp<P, N>>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &Self) -> Self {
        use core::ops::AddAssign;
        self.add_assign(other);
        self
    }
}

impl<'a, P: FpParams<N>, const N: usize> core::ops::Sub<&'a Fp<P, N>>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &Self) -> Self {
        use core::ops::SubAssign;
        self.sub_assign(other);
        self
    }
}

impl<'a, P: FpParams<N>, const N: usize> core::ops::Mul<&'a Fp<P, N>>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(other);
        self
    }
}

impl<'a, P: FpParams<N>, const N: usize> core::ops::Div<&'a Fp<P, N>>
    for Fp<P, N>
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

impl<'a, P: FpParams<N>, const N: usize> core::ops::AddAssign<&'a Self>
    for Fp<P, N>
{
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        P::add_assign(self, other);
    }
}

impl<'a, P: FpParams<N>, const N: usize> core::ops::SubAssign<&'a Self>
    for Fp<P, N>
{
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
impl<'a, P: FpParams<N>, const N: usize> core::ops::Add<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &'a mut Self) -> Self {
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
impl<'a, P: FpParams<N>, const N: usize> core::ops::Sub<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &'a mut Self) -> Self {
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
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::SubAssign<Self> for Fp<P, N> {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::ops::AddAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn add_assign(&mut self, other: &'a mut Self) {
        self.add_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::ops::SubAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn sub_assign(&mut self, other: &'a mut Self) {
        self.sub_assign(&*other);
    }
}

impl<'a, P: FpParams<N>, const N: usize> core::ops::MulAssign<&'a Self>
    for Fp<P, N>
{
    fn mul_assign(&mut self, other: &Self) {
        P::mul_assign(self, other);
    }
}

/// Computes `self *= other.inverse()` if `other.inverse()` is `Some`, and
/// panics otherwise.
impl<'a, P: FpParams<N>, const N: usize> core::ops::DivAssign<&'a Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn div_assign(&mut self, other: &Self) {
        use core::ops::MulAssign;
        self.mul_assign(&other.inverse().unwrap());
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Mul<Self> for Fp<P, N> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::Div<Self> for Fp<P, N> {
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: Self) -> Self {
        use core::ops::DivAssign;
        self.div_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::ops::Mul<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: &'a mut Self) -> Self {
        use core::ops::MulAssign;
        self.mul_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::ops::Div<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: &'a mut Self) -> Self {
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
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        self.mul_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::ops::DivAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn div_assign(&mut self, other: &'a mut Self) {
        self.div_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> core::ops::MulAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn mul_assign(&mut self, other: &'a mut Self) {
        self.mul_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> core::ops::DivAssign<Self> for Fp<P, N> {
    #[inline(always)]
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

// TODO#q: add feature num_bigint
// impl<P: FpConfig<N>, const N: usize> From<num_bigint::BigUint> for Fp<P, N> {
//     #[inline]
//     fn from(val: num_bigint::BigUint) -> Fp<P, N> {
//         Fp::<P, N>::from_le_bytes_mod_order(&val.to_bytes_le())
//     }
// }
//
// impl<P: FpConfig<N>, const N: usize> From<Fp<P, N>> for num_bigint::BigUint {
//     #[inline(always)]
//     fn from(other: Fp<P, N>) -> Self {
//         other.into_bigint().into()
//     }
// }

impl<P: FpParams<N>, const N: usize> From<Fp<P, N>> for Uint<N> {
    #[inline(always)]
    fn from(fp: Fp<P, N>) -> Self {
        fp.into_bigint()
    }
}

impl<P: FpParams<N>, const N: usize> From<Uint<N>> for Fp<P, N> {
    /// Converts `Self::BigInteger` into `Self`
    #[inline(always)]
    fn from(int: Uint<N>) -> Self {
        Self::from_bigint(int).expect("should be less than modulus")
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
