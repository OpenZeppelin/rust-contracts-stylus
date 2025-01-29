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
    iter::{Product, Sum},
    marker::PhantomData,
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
    },
};

use educe::Educe;
use num_traits::{One, Zero};

use crate::{
    arithmetic,
    arithmetic::{
        limb,
        uint::{Uint, WideUint},
        BigInteger,
    },
    ct_for, ct_for_unroll6,
    field::{group::AdditiveGroup, prime::PrimeField, Field},
};

/// A trait that specifies the configuration of a prime field.
/// Also specifies how to perform arithmetic on field elements.
pub trait FpParams<const N: usize>: Send + Sync + 'static + Sized {
    /// The modulus of the field.
    const MODULUS: Uint<N>;

    /// A multiplicative generator of the field.
    /// [`Self::GENERATOR`] is an element having multiplicative order
    /// `MODULUS - 1`.
    const GENERATOR: Fp<Self, N>;

    /// `MODULUS` has a spare bit in the most significant limb.
    const HAS_MODULUS_SPARE_BIT: bool = Self::MODULUS.limbs[N - 1] >> 63 == 0;

    /// `INV = -MODULUS^{-1} mod 2^64`
    const INV: u64 = inv::<Self, N>();

    /// Let `M` be the power of 2^64 nearest to [`Self::MODULUS`] size.
    ///
    /// Then `R = M % MODULUS` or `R = (M - 1) % MODULUS + 1` for convenience of
    /// multiplication.
    const R: Uint<N> = WideUint::new(Uint::<N>::MAX, Uint::<N>::ZERO)
        .ct_rem(&Self::MODULUS)
        .ct_wrapping_add(&Uint::ONE);

    /// `R2 = R^2 % MODULUS` or `R2 = (R^2 - 1) % MODULUS + 1` for convenience
    /// of multiplication.
    #[allow(dead_code)]
    const R2: Uint<N> = WideUint::new(Uint::<N>::MAX, Uint::<N>::MAX)
        .ct_rem(&Self::MODULUS)
        .ct_wrapping_add(&Uint::ONE);

    /// Set `a += b`.
    #[inline(always)]
    fn add_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        // This cannot exceed the backing capacity.
        let c = a.montgomery_form.checked_add_assign(&b.montgomery_form);
        // However, it may need to be reduced
        if Self::HAS_MODULUS_SPARE_BIT {
            a.subtract_modulus();
        } else {
            a.carrying_sub_modulus(c);
        }
    }

    /// Set `a -= b`.
    #[inline(always)]
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        // If `other` is larger than `self`, add the modulus to self first.
        if b.montgomery_form > a.montgomery_form {
            a.montgomery_form.checked_add_assign(&Self::MODULUS);
        }
        a.montgomery_form.checked_sub_assign(&b.montgomery_form);
    }

    /// Set `a = a + a`.
    #[inline(always)]
    fn double_in_place(a: &mut Fp<Self, N>) {
        // This cannot exceed the backing capacity.
        let c = a.montgomery_form.checked_mul2_assign();
        // However, it may need to be reduced.
        if Self::HAS_MODULUS_SPARE_BIT {
            a.subtract_modulus();
        } else {
            a.carrying_sub_modulus(c);
        }
    }

    /// Set `a = -a`;
    #[inline(always)]
    fn neg_in_place(a: &mut Fp<Self, N>) {
        if !a.is_zero() {
            let mut tmp = Self::MODULUS;
            tmp.checked_sub_assign(&a.montgomery_form);
            a.montgomery_form = tmp;
        }
    }

    /// Set `a *= b`.
    ///
    /// This modular multiplication algorithm uses Montgomery
    /// reduction for efficient implementation.
    #[inline(always)]
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        // Implements CIOS.
        let (carry, res) = a.ct_mul_without_cond_subtract(b);
        *a = res;

        if Self::HAS_MODULUS_SPARE_BIT {
            a.subtract_modulus();
        } else {
            a.carrying_sub_modulus(carry);
        }
    }

    /// Set `a *= a`.
    #[inline(always)]
    fn square_in_place(a: &mut Fp<Self, N>) {
        Self::mul_assign(a, &a.clone());
    }

    /// Compute `a^{-1}` if `a` is not zero.
    ///
    /// Guajardo, Kumar, Paar, Pelzl.
    /// Efficient Software-Implementation of Finite Fields with Applications to
    /// Cryptography [reference].
    /// Algorithm 16 (BEA for Inversion in Fp).
    ///
    /// [reference]: https://www.sandeep.de/my/papers/2006_ActaApplMath_EfficientSoftFiniteF.pdf
    #[must_use]
    #[inline(always)]
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        if a.is_zero() {
            return None;
        }

        let one = Uint::ONE;

        let mut u = a.montgomery_form;
        let mut v = Self::MODULUS;
        let mut b = Fp::new_unchecked(Self::R2); // Avoids unnecessary reduction step.
        let mut c = Fp::zero();

        while u != one && v != one {
            while u.is_even() {
                u.div2_assign();

                if b.montgomery_form.is_even() {
                    b.montgomery_form.div2_assign();
                } else {
                    let carry =
                        b.montgomery_form.checked_add_assign(&Self::MODULUS);
                    b.montgomery_form.div2_assign();
                    if !Self::HAS_MODULUS_SPARE_BIT && carry {
                        b.montgomery_form.limbs[N - 1] |= 1 << 63;
                    }
                }
            }

            while v.is_even() {
                v.div2_assign();

                if c.montgomery_form.is_even() {
                    c.montgomery_form.div2_assign();
                } else {
                    let carry =
                        c.montgomery_form.checked_add_assign(&Self::MODULUS);
                    c.montgomery_form.div2_assign();
                    if !Self::HAS_MODULUS_SPARE_BIT && carry {
                        c.montgomery_form.limbs[N - 1] |= 1 << 63;
                    }
                }
            }

            if v < u {
                u.checked_sub_assign(&v);
                b -= &c;
            } else {
                v.checked_sub_assign(&u);
                c -= &b;
            }
        }

        if u == one {
            Some(b)
        } else {
            Some(c)
        }
    }

    /// Construct a field element from an integer.
    ///
    /// By the end element will be converted to a montgomery form and reduced.
    #[must_use]
    #[inline(always)]
    fn from_bigint(num: Uint<N>) -> Fp<Self, N> {
        let elem = Fp::new_unchecked(num);
        if elem.is_zero() {
            elem
        } else {
            elem * Fp::new_unchecked(Self::R2)
        }
    }

    /// Convert a field element to an integer less than [`Self::MODULUS`].
    #[must_use]
    #[inline(always)]
    fn into_bigint(elem: Fp<Self, N>) -> Uint<N> {
        elem.montgomery_reduction()
    }
}

/// Compute `-M^{-1} mod 2^64`.
pub const fn inv<T: FpParams<N>, const N: usize>() -> u64 {
    // We compute this as follows.
    // First, MODULUS mod 2^64 is just the lower 64 bits of MODULUS.
    // Hence MODULUS mod 2^64 = MODULUS.0[0] mod 2^64.
    //
    // Next, computing the inverse mod 2^64 involves exponentiating by
    // the multiplicative group order, which is euler_totient(2^64) - 1.
    // Now, euler_totient(2^64) = 1 << 63, and so
    // euler_totient(2^64) - 1 = (1 << 63) - 1 = 1111111... (63 digits).
    // We compute this powering via standard square and multiply.
    let mut inv = 1u64;
    ct_for!((_i in 0..63) {
        // Square
        inv = inv.wrapping_mul(inv);
        // Multiply
        inv = inv.wrapping_mul(T::MODULUS.limbs[0]);
    });
    inv.wrapping_neg()
}

/// Represents an element of the prime field `F_p`, where `p == P::MODULUS`.
///
/// This type can represent elements in any field of size at most N * 64 bits.
#[derive(Educe)]
#[educe(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fp<P: FpParams<N>, const N: usize> {
    /// Contains the element in Montgomery form for efficient multiplication.
    /// To convert an element to a [`Uint`], use [`FpParams::into_bigint`]
    /// or `into`.
    montgomery_form: Uint<N>,
    #[doc(hidden)]
    phantom: PhantomData<P>,
}

/// Declare [`Fp`] types for different bit sizes.
macro_rules! declare_fp {
    ($fp:ident, $limbs:ident, $bits:expr) => {
        #[doc = "Finite field with max"]
        #[doc = stringify!($bits)]
        #[doc = "bits size element."]
        pub type $fp<P> = $crate::field::fp::Fp<
            P,
            {
                usize::div_ceil(
                    $bits,
                    $crate::arithmetic::limb::Limb::BITS as usize,
                )
            },
        >;

        #[doc = "Number of limbs in the field with"]
        #[doc = stringify!($bits)]
        #[doc = "bits size element."]
        pub const $limbs: usize = usize::div_ceil(
            $bits,
            $crate::arithmetic::limb::Limb::BITS as usize,
        );
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

impl<P: FpParams<N>, const N: usize> Fp<P, N> {
    /// A multiplicative generator of the field.
    /// [`Self::GENERATOR`] is an element having multiplicative order
    /// `MODULUS - 1`.
    ///
    /// Every element of the field should be represented as `GENERATOR^i`
    pub const GENERATOR: Fp<P, N> = P::GENERATOR;
    /// Multiplicative identity of the field, i.e., the element `e`
    /// such that, for all elements `f` of the field, `e * f = f`.
    pub const ONE: Fp<P, N> = Fp::new_unchecked(P::R);
    /// Additive identity of the field, i.e., the element `e`
    /// such that, for all elements `f` of the field, `e + f = f`.
    pub const ZERO: Fp<P, N> = Fp::new_unchecked(Uint { limbs: [0; N] });

    /// Construct a new field element from [`Uint`].
    ///
    /// Unlike [`Self::new`], this method does not perform Montgomery reduction.
    /// This method should be used only when constructing an element from an
    /// integer that has already been put in Montgomery form.
    #[must_use]
    #[inline(always)]
    pub const fn new_unchecked(element: Uint<N>) -> Self {
        Self { montgomery_form: element, phantom: PhantomData }
    }

    #[doc(hidden)]
    #[inline(always)]
    #[must_use]
    pub fn is_geq_modulus(&self) -> bool {
        self.montgomery_form >= P::MODULUS
    }

    #[inline(always)]
    fn subtract_modulus(&mut self) {
        if self.is_geq_modulus() {
            self.montgomery_form.checked_sub_assign(&Self::MODULUS);
        }
    }

    /// Construct a new field element from its underlying
    /// [`struct@Uint`] data type.
    #[inline]
    #[must_use]
    pub const fn new(element: Uint<N>) -> Self {
        let mut r = Self { montgomery_form: element, phantom: PhantomData };
        if r.ct_is_zero() {
            r
        } else {
            r = r.ct_mul(&Fp { montgomery_form: P::R2, phantom: PhantomData });
            r
        }
    }

    /// Multiply `self` to `rhs` and return the result (constant).
    const fn ct_mul(&self, rhs: &Self) -> Self {
        let (carry, res) = self.ct_mul_without_cond_subtract(rhs);
        if P::HAS_MODULUS_SPARE_BIT {
            res.ct_subtract_modulus()
        } else {
            res.ct_carrying_sub_modulus(carry)
        }
    }

    //// Returns true if this number is zero (constant).
    const fn ct_is_zero(&self) -> bool {
        self.montgomery_form.ct_is_zero()
    }

    /// Subtract modulus from `self` with carry.
    #[inline(always)]
    fn carrying_sub_modulus(&mut self, carry: bool) {
        if carry || self.is_geq_modulus() {
            self.montgomery_form.checked_sub_assign(&Self::MODULUS);
        }
    }

    #[inline(always)]
    const fn ct_mul_without_cond_subtract(&self, other: &Self) -> (bool, Self) {
        let (lo, hi) =
            self.montgomery_form.ct_widening_mul(&other.montgomery_form);

        let (carry, res) = Self::widening_montgomery_reduction(lo, hi);
        (carry, Self::new_unchecked(res))
    }

    /// Apply the Montgomery reduction to composite number represented as `lo`
    /// and `hi`.
    /// Returns `carry` and the result of the reduction.
    ///
    /// Algorithm 14.32 in Handbook of Applied Cryptography [reference].
    ///
    /// [reference]: https://cacr.uwaterloo.ca/hac/about/chap14.pdf
    #[inline(always)]
    const fn widening_montgomery_reduction(
        mut lo: Uint<N>,
        mut hi: Uint<N>,
    ) -> (bool, Uint<N>) {
        let mut carry2 = 0;
        ct_for_unroll6!((i in 0..N) {
            let tmp = lo.limbs[i].wrapping_mul(P::INV);

            let (_, mut carry) = arithmetic::limb::mac(lo.limbs[i], tmp, P::MODULUS.limbs[0]);

            ct_for_unroll6!((j in 1..N) {
                let k = i + j;
                if k >= N {
                    (hi.limbs[k - N], carry) = arithmetic::limb::carrying_mac(
                        hi.limbs[k - N],
                        tmp,
                        P::MODULUS.limbs[j],
                        carry
                    );
                } else {
                    (lo.limbs[k], carry) = arithmetic::limb::carrying_mac(
                        lo.limbs[k],
                        tmp,
                        P::MODULUS.limbs[j],
                        carry
                    );
                }
            });
            (hi.limbs[i], carry2) = arithmetic::limb::adc(hi.limbs[i], carry, carry2);
        });

        (carry2 != 0, hi)
    }

    /// Apply the Montgomery reduction to `self`.
    /// Returns the result of the reduction (carry not needed).
    /// Compare to [`Self::widening_montgomery_reduction`] doesn't use loop
    /// "unroll" optimization, since it is assumed to be called just to
    /// convert back to normal representation.
    ///
    /// Algorithm 14.32 in Handbook of Applied Cryptography [reference].
    ///
    /// [reference]: https://cacr.uwaterloo.ca/hac/about/chap14.pdf
    #[inline(always)]
    fn montgomery_reduction(self) -> Uint<N> {
        let mut limbs = self.montgomery_form.limbs;
        for i in 0..N {
            let k = limbs[i].wrapping_mul(P::INV);

            let (_, mut carry) = limb::mac(limbs[i], k, Self::MODULUS.limbs[0]);
            for j in 1..N {
                (limbs[(j + i) % N], carry) = limb::carrying_mac(
                    limbs[(j + i) % N],
                    k,
                    Self::MODULUS.limbs[j],
                    carry,
                );
            }
            limbs[i % N] = carry;
        }
        Uint::new(limbs)
    }

    const fn ct_is_valid(&self) -> bool {
        ct_for!((i in 0..N) {
            if self.montgomery_form.limbs[N - i - 1] < P::MODULUS.limbs[N - i - 1] {
                return true
            } else if self.montgomery_form.limbs[N - i - 1] > P::MODULUS.limbs[N - i - 1] {
                return false
            }
        });
        false
    }

    #[inline]
    const fn ct_subtract_modulus(mut self) -> Self {
        if !self.ct_is_valid() {
            self.montgomery_form =
                Self::sub_with_borrow(&self.montgomery_form, &P::MODULUS);
        }
        self
    }

    #[inline]
    const fn ct_carrying_sub_modulus(mut self, carry: bool) -> Self {
        if carry || !self.ct_is_valid() {
            self.montgomery_form =
                Self::sub_with_borrow(&self.montgomery_form, &P::MODULUS);
        }
        self
    }

    const fn sub_with_borrow(a: &Uint<N>, b: &Uint<N>) -> Uint<N> {
        a.ct_checked_sub(b).0
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
        Self::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl<P: FpParams<N>, const N: usize> One for Fp<P, N> {
    #[inline]
    fn one() -> Self {
        Self::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
}

impl<P: FpParams<N>, const N: usize> AdditiveGroup for Fp<P, N> {
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

impl<P: FpParams<N>, const N: usize> Field for Fp<P, N> {
    const ONE: Self = Fp::new_unchecked(P::R);

    #[inline]
    fn square(&self) -> Self {
        let mut temp = *self;
        temp.square_in_place();
        temp
    }

    #[inline]
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
    const MODULUS_BIT_SIZE: usize = <Uint<N> as BigInteger>::BITS;

    #[inline]
    fn from_bigint(repr: Self::BigInt) -> Self {
        P::from_bigint(repr)
    }

    #[inline]
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

/// Auto implements conversion from unsigned integer of type `$int` to [`Fp`].
macro_rules! impl_fp_from_unsigned_int {
    ($int:ty) => {
        impl<P: FpParams<N>, const N: usize> From<$int> for Fp<P, N> {
            fn from(other: $int) -> Self {
                Fp::from_bigint(Uint::from(other))
            }
        }
    };
}

/// Auto implements conversion from signed integer of type `$int` to [`Fp`].
macro_rules! impl_fp_from_signed_int {
    ($int:ty) => {
        impl<P: FpParams<N>, const N: usize> From<$int> for Fp<P, N> {
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

impl<P: FpParams<N>, const N: usize> From<bool> for Fp<P, N> {
    fn from(other: bool) -> Self {
        u8::from(other).into()
    }
}

/// Auto implements conversion from [`Fp`] to integer of type `$int`.
///
/// Conversion is available only for a single limb field elements,
/// i.e. `N = 1`.
macro_rules! impl_int_from_fp {
    ($int:ty) => {
        impl<P: FpParams<1>> From<Fp<P, 1>> for $int {
            fn from(other: Fp<P, 1>) -> Self {
                let uint = other.into_bigint();
                let words = uint.as_limbs();
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

/// Outputs a string containing the value of `self`,
/// represented as a decimal without leading zeroes.
impl<P: FpParams<N>, const N: usize> Display for Fp<P, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let str = self.into_bigint().to_string();
        write!(f, "{str}")
    }
}

impl<P: FpParams<N>, const N: usize> Neg for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn neg(mut self) -> Self {
        P::neg_in_place(&mut self);
        self
    }
}

impl<P: FpParams<N>, const N: usize> Add<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &Self) -> Self {
        use AddAssign;
        self.add_assign(other);
        self
    }
}

impl<P: FpParams<N>, const N: usize> Sub<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &Self) -> Self {
        use SubAssign;
        self.sub_assign(other);
        self
    }
}

impl<P: FpParams<N>, const N: usize> Mul<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &Self) -> Self {
        use MulAssign;
        self.mul_assign(other);
        self
    }
}

impl<P: FpParams<N>, const N: usize> Div<&Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    /// Returns `self * other.inverse()` if `other.inverse()` is `Some`, and
    /// panics otherwise.
    #[inline]
    fn div(mut self, other: &Self) -> Self {
        use MulAssign;
        self.mul_assign(&other.inverse().unwrap());
        self
    }
}

impl<P: FpParams<N>, const N: usize> Add<&Fp<P, N>> for &Fp<P, N> {
    type Output = Fp<P, N>;

    #[inline]
    fn add(self, other: &Fp<P, N>) -> Fp<P, N> {
        use AddAssign;
        let mut result = *self;
        result.add_assign(other);
        result
    }
}

impl<P: FpParams<N>, const N: usize> Sub<&Fp<P, N>> for &Fp<P, N> {
    type Output = Fp<P, N>;

    #[inline]
    fn sub(self, other: &Fp<P, N>) -> Fp<P, N> {
        use SubAssign;
        let mut result = *self;
        result.sub_assign(other);
        result
    }
}

impl<P: FpParams<N>, const N: usize> Mul<&Fp<P, N>> for &Fp<P, N> {
    type Output = Fp<P, N>;

    #[inline]
    fn mul(self, other: &Fp<P, N>) -> Fp<P, N> {
        use MulAssign;
        let mut result = *self;
        result.mul_assign(other);
        result
    }
}

impl<P: FpParams<N>, const N: usize> Div<&Fp<P, N>> for &Fp<P, N> {
    type Output = Fp<P, N>;

    #[inline]
    fn div(self, other: &Fp<P, N>) -> Fp<P, N> {
        use DivAssign;
        let mut result = *self;
        result.div_assign(other);
        result
    }
}

impl<P: FpParams<N>, const N: usize> AddAssign<&Self> for Fp<P, N> {
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        P::add_assign(self, other);
    }
}

impl<P: FpParams<N>, const N: usize> SubAssign<&Self> for Fp<P, N> {
    #[inline]
    fn sub_assign(&mut self, other: &Self) {
        P::sub_assign(self, other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Add<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: Self) -> Self {
        use AddAssign;
        self.add_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Add<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &mut Self) -> Self {
        use AddAssign;
        self.add_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Sub<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: Self) -> Self {
        use SubAssign;
        self.sub_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Sub<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &mut Self) -> Self {
        use SubAssign;
        self.sub_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Sum<Self> for Fp<P, N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> Sum<&'a Self> for Fp<P, N> {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> AddAssign<Self> for Fp<P, N> {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> SubAssign<Self> for Fp<P, N> {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> AddAssign<&mut Self> for Fp<P, N> {
    #[inline]
    fn add_assign(&mut self, other: &mut Self) {
        self.add_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> SubAssign<&mut Self> for Fp<P, N> {
    #[inline]
    fn sub_assign(&mut self, other: &mut Self) {
        self.sub_assign(&*other);
    }
}

impl<P: FpParams<N>, const N: usize> MulAssign<&Self> for Fp<P, N> {
    #[inline]
    fn mul_assign(&mut self, other: &Self) {
        P::mul_assign(self, other);
    }
}

/// Computes `self *= other.inverse()` if `other.inverse()` is `Some`, and
/// panics otherwise.
impl<P: FpParams<N>, const N: usize> DivAssign<&Self> for Fp<P, N> {
    #[inline]
    fn div_assign(&mut self, other: &Self) {
        use MulAssign;
        self.mul_assign(&other.inverse().expect("should not divide by zero"));
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Mul<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: Self) -> Self {
        use MulAssign;
        self.mul_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Div<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn div(mut self, other: Self) -> Self {
        use DivAssign;
        self.div_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Mul<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &mut Self) -> Self {
        use MulAssign;
        self.mul_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Div<&mut Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn div(mut self, other: &mut Self) -> Self {
        use DivAssign;
        self.div_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> Product<Self> for Fp<P, N> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpParams<N>, const N: usize> Product<&'a Self> for Fp<P, N> {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> MulAssign<Self> for Fp<P, N> {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        self.mul_assign(&other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> DivAssign<&mut Self> for Fp<P, N> {
    #[inline]
    fn div_assign(&mut self, other: &mut Self) {
        self.div_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> MulAssign<&mut Self> for Fp<P, N> {
    #[inline]
    fn mul_assign(&mut self, other: &mut Self) {
        self.mul_assign(&*other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpParams<N>, const N: usize> DivAssign<Self> for Fp<P, N> {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        self.div_assign(&other);
    }
}

impl<P: FpParams<N>, const N: usize> zeroize::Zeroize for Fp<P, N> {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        self.montgomery_form.zeroize();
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
        $crate::field::fp::Fp::new($crate::arithmetic::uint::from_str_radix(
            $num, 10,
        ))
    };
}

/// This macro converts a string hex number to a field element.
#[macro_export]
macro_rules! fp_from_hex {
    ($num:literal) => {{
        $crate::field::fp::Fp::new($crate::arithmetic::uint::from_str_hex($num))
    }};
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::{
        arithmetic::uint::U64,
        field::{
            fp::{Fp64, FpParams, LIMBS_64},
            group::AdditiveGroup,
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
            let a = i128::from(a);
            let b = i128::from(b);
            prop_assert_eq!(res, (a + b).rem_euclid(MODULUS));
        }

        #[test]
        fn double(a: i64) {
            let res = Field64::from(a).double();
            let res: i128 = res.into();
            let a = i128::from(a);
            prop_assert_eq!(res, (a + a).rem_euclid(MODULUS));
        }

        #[test]
        fn sub(a: i64, b: i64) {
            let res = Field64::from(a) - Field64::from(b);
            let res: i128 = res.into();
            let a = i128::from(a);
            let b = i128::from(b);
            prop_assert_eq!(res, (a - b).rem_euclid(MODULUS));
        }

        #[test]
        fn mul(a: i64, b: i64) {
            let res = Field64::from(a) * Field64::from(b);
            let res: i128 = res.into();
            let a = i128::from(a);
            let b = i128::from(b);
            prop_assert_eq!(res, (a * b).rem_euclid(MODULUS));
        }

        #[test]
        fn square(a: i64) {
            let res = Field64::from(a).square();
            let res: i128 = res.into();
            let a = i128::from(a);
            prop_assert_eq!(res, (a * a).rem_euclid(MODULUS));
        }

        #[test]
        fn div(a: i64, b: i64) {
            // Skip if `b` is zero.
            if i128::from(b) % MODULUS == 0 {
                return Ok(());
            }

            let res = Field64::from(a) / Field64::from(b);
            let res: i128 = res.into();
            let a = i128::from(a);
            let b = i128::from(b);
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
            let a = i128::from(a);
            let b = i128::from(b);
            prop_assert_eq!(res, dumb_pow(a, b));
        }

        #[test]
        fn neg(a: i64) {
            let res = -Field64::from(a);
            let res: i128 = res.into();
            let a = i128::from(a);
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
