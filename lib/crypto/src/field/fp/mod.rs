use core::iter;

use ark_serialize::{
    buffer_byte_size, CanonicalDeserialize, CanonicalDeserializeWithFlags,
    CanonicalSerialize, CanonicalSerializeWithFlags, Compress, EmptyFlags,
    Flags, SerializationError, Valid, Validate,
};
use ark_std::{
    cmp::*,
    fmt::{Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign,
    },
    str::FromStr,
    string::*,
};
use educe::Educe;
use num_traits::{One, Zero};

use crate::{
    biginteger::{BigInt, BigInteger},
    field::{prime::PrimeField, AdditiveGroup, Field},
};

/// A trait that specifies the configuration of a prime field.
/// Also specifies how to perform arithmetic on field elements.
pub trait FpConfig<const N: usize>: Send + Sync + 'static + Sized {
    /// The modulus of the field.
    const MODULUS: BigInt<N>;

    /// A multiplicative generator of the field.
    /// `Self::GENERATOR` is an element having multiplicative order
    /// `Self::MODULUS - 1`.
    const GENERATOR: Fp<Self, N>;

    /// Additive identity of the field, i.e. the element `e`
    /// such that, for all elements `f` of the field, `e + f = f`.
    const ZERO: Fp<Self, N> = Fp::new_unchecked(BigInt([0u64; N]));

    /// Multiplicative identity of the field, i.e. the element `e`
    /// such that, for all elements `f` of the field, `e * f = f`.
    const ONE: Fp<Self, N> = Fp::new_unchecked(Self::R);

    /// Let `M` be the power of 2^64 nearest to `Self::MODULUS_BITS`. Then
    /// `R = M % Self::MODULUS`.
    const R: BigInt<N> = Self::MODULUS.montgomery_r();

    /// R2 = R^2 % Self::MODULUS
    const R2: BigInt<N> = Self::MODULUS.montgomery_r2();

    /// INV = -MODULUS^{-1} mod 2^64
    const INV: u64 = inv::<Self, N>();

    /// Can we use the no-carry optimization for multiplication
    /// outlined [here](https://hackmd.io/@gnark/modular_multiplication)?
    ///
    /// This optimization applies if
    /// (a) `Self::MODULUS[N-1] < u64::MAX >> 1`, and
    /// (b) the bits of the modulus are not all 1.
    #[doc(hidden)]
    const CAN_USE_NO_CARRY_MUL_OPT: bool =
        can_use_no_carry_mul_optimization::<Self, N>();

    /// Can we use the no-carry optimization for squaring
    /// outlined [here](https://hackmd.io/@gnark/modular_multiplication)?
    ///
    /// This optimization applies if
    /// (a) `Self::MODULUS[N-1] < u64::MAX >> 2`, and
    /// (b) the bits of the modulus are not all 1.
    #[doc(hidden)]
    const CAN_USE_NO_CARRY_SQUARE_OPT: bool =
        can_use_no_carry_mul_optimization::<Self, N>();

    /// Does the modulus have a spare unused bit
    ///
    /// This condition applies if
    /// (a) `Self::MODULUS[N-1] >> 63 == 0`
    #[doc(hidden)]
    const MODULUS_HAS_SPARE_BIT: bool = modulus_has_spare_bit::<Self, N>();

    /// Set a += b.
    fn add_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        // This cannot exceed the backing capacity.
        let c = a.0.add_with_carry(&b.0);
        // However, it may need to be reduced
        if Self::MODULUS_HAS_SPARE_BIT {
            a.subtract_modulus()
        } else {
            a.subtract_modulus_with_carry(c)
        }
    }

    /// Set a -= b.
    fn sub_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        // If `other` is larger than `self`, add the modulus to self first.
        if b.0 > a.0 {
            a.0.add_with_carry(&Self::MODULUS);
        }
        a.0.sub_with_borrow(&b.0);
    }

    /// Set a = a + a.
    fn double_in_place(a: &mut Fp<Self, N>) {
        // This cannot exceed the backing capacity.
        let c = a.0.mul2();
        // However, it may need to be reduced.
        if Self::MODULUS_HAS_SPARE_BIT {
            a.subtract_modulus()
        } else {
            a.subtract_modulus_with_carry(c)
        }
    }

    /// Set a = -a;
    fn neg_in_place(a: &mut Fp<Self, N>) {
        if !a.is_zero() {
            let mut tmp = Self::MODULUS;
            tmp.sub_with_borrow(&a.0);
            a.0 = tmp;
        }
    }

    /// Set a *= b.
    fn mul_assign(a: &mut Fp<Self, N>, b: &Fp<Self, N>) {
        // No-carry optimisation applied to CIOS
        if Self::CAN_USE_NO_CARRY_MUL_OPT {
            if N <= 6
                && N > 1
                && cfg!(all(
                    feature = "asm",
                    target_feature = "bmi2",
                    target_feature = "adx",
                    target_arch = "x86_64"
                ))
            {
                #[cfg(
                    all(
                        feature = "asm",
                        target_feature = "bmi2",
                        target_feature = "adx",
                        target_arch = "x86_64"
                    )
                )]
                #[allow(unsafe_code, unused_mut)]
                #[rustfmt::skip]

                // Tentatively avoid using assembly for `N == 1`.
                match N {
                    2 => { ark_ff_asm::x86_64_asm_mul!(2, (a.0).0, (b.0).0); },
                    3 => { ark_ff_asm::x86_64_asm_mul!(3, (a.0).0, (b.0).0); },
                    4 => { ark_ff_asm::x86_64_asm_mul!(4, (a.0).0, (b.0).0); },
                    5 => { ark_ff_asm::x86_64_asm_mul!(5, (a.0).0, (b.0).0); },
                    6 => { ark_ff_asm::x86_64_asm_mul!(6, (a.0).0, (b.0).0); },
                    _ => unsafe { ark_std::hint::unreachable_unchecked() },
                };
            } else {
                let mut r = [0u64; N];

                for i in 0..N {
                    let mut carry1 = 0u64;
                    r[0] = crate::biginteger::arithmetic::mac(
                        r[0],
                        (a.0).0[0],
                        (b.0).0[i],
                        &mut carry1,
                    );

                    let k = r[0].wrapping_mul(Self::INV);

                    let mut carry2 = 0u64;
                    crate::biginteger::arithmetic::mac_discard(
                        r[0],
                        k,
                        Self::MODULUS.0[0],
                        &mut carry2,
                    );

                    for j in 1..N {
                        r[j] = crate::biginteger::arithmetic::mac_with_carry(
                            r[j],
                            (a.0).0[j],
                            (b.0).0[i],
                            &mut carry1,
                        );
                        r[j - 1] =
                            crate::biginteger::arithmetic::mac_with_carry(
                                r[j],
                                k,
                                Self::MODULUS.0[j],
                                &mut carry2,
                            );
                    }
                    r[N - 1] = carry1 + carry2;
                }
                (a.0).0.copy_from_slice(&r);
            }
            a.subtract_modulus();
        } else {
            // Alternative implementation
            // Implements CIOS.
            let (carry, res) = a.mul_without_cond_subtract(b);
            *a = res;

            if Self::MODULUS_HAS_SPARE_BIT {
                a.subtract_modulus_with_carry(carry);
            } else {
                a.subtract_modulus();
            }
        }
    }

    /// Set a *= a.
    fn square_in_place(a: &mut Fp<Self, N>) {
        if N == 1 {
            // We default to multiplying with `a` using the `Mul` impl
            // for the N == 1 case
            *a *= *a;
            return;
        }
        if Self::CAN_USE_NO_CARRY_SQUARE_OPT
            && (2..=6).contains(&N)
            && cfg!(all(
                feature = "asm",
                target_feature = "bmi2",
                target_feature = "adx",
                target_arch = "x86_64"
            ))
        {
            #[cfg(all(
                feature = "asm",
                target_feature = "bmi2",
                target_feature = "adx",
                target_arch = "x86_64"
            ))]
            #[allow(unsafe_code, unused_mut)]
            #[rustfmt::skip]
            match N {
                2 => { ark_ff_asm::x86_64_asm_square!(2, (a.0).0); },
                3 => { ark_ff_asm::x86_64_asm_square!(3, (a.0).0); },
                4 => { ark_ff_asm::x86_64_asm_square!(4, (a.0).0); },
                5 => { ark_ff_asm::x86_64_asm_square!(5, (a.0).0); },
                6 => { ark_ff_asm::x86_64_asm_square!(6, (a.0).0); },
                _ => unsafe { ark_std::hint::unreachable_unchecked() },
            };
            a.subtract_modulus();
            return;
        }

        let mut r = crate::const_helpers::MulBuffer::<N>::zeroed();

        let mut carry = 0;
        for i in 0..(N - 1) {
            for j in (i + 1)..N {
                r[i + j] = crate::biginteger::arithmetic::mac_with_carry(
                    r[i + j],
                    (a.0).0[i],
                    (a.0).0[j],
                    &mut carry,
                );
            }
            r.b1[i] = carry;
            carry = 0;
        }

        r.b1[N - 1] = r.b1[N - 2] >> 63;
        for i in 2..(2 * N - 1) {
            r[2 * N - i] = (r[2 * N - i] << 1) | (r[2 * N - (i + 1)] >> 63);
        }
        r.b0[1] <<= 1;

        for i in 0..N {
            r[2 * i] = crate::biginteger::arithmetic::mac_with_carry(
                r[2 * i],
                (a.0).0[i],
                (a.0).0[i],
                &mut carry,
            );
            carry =
                crate::biginteger::arithmetic::adc(&mut r[2 * i + 1], 0, carry);
        }
        // Montgomery reduction
        let mut carry2 = 0;
        for i in 0..N {
            let k = r[i].wrapping_mul(Self::INV);
            carry = 0;
            crate::biginteger::arithmetic::mac_discard(
                r[i],
                k,
                Self::MODULUS.0[0],
                &mut carry,
            );
            for j in 1..N {
                r[j + i] = crate::biginteger::arithmetic::mac_with_carry(
                    r[j + i],
                    k,
                    Self::MODULUS.0[j],
                    &mut carry,
                );
            }
            carry2 =
                crate::biginteger::arithmetic::adc(&mut r.b1[i], carry, carry2);
        }
        (a.0).0.copy_from_slice(&r.b1);
        if Self::MODULUS_HAS_SPARE_BIT {
            a.subtract_modulus();
        } else {
            a.subtract_modulus_with_carry(carry2 != 0);
        }
    }

    /// Compute a^{-1} if `a` is not zero.
    fn inverse(a: &Fp<Self, N>) -> Option<Fp<Self, N>> {
        if a.is_zero() {
            return None;
        }
        // Guajardo Kumar Paar Pelzl
        // Efficient Software-Implementation of Finite Fields with Applications
        // to Cryptography
        // Algorithm 16 (BEA for Inversion in Fp)

        let one = BigInt::from(1u64);

        let mut u = a.0;
        let mut v = Self::MODULUS;
        let mut b = Fp::new_unchecked(Self::R2); // Avoids unnecessary reduction step.
        let mut c = Fp::zero();

        while u != one && v != one {
            while u.is_even() {
                u.div2();

                if b.0.is_even() {
                    b.0.div2();
                } else {
                    let carry = b.0.add_with_carry(&Self::MODULUS);
                    b.0.div2();
                    if !Self::MODULUS_HAS_SPARE_BIT && carry {
                        (b.0).0[N - 1] |= 1 << 63;
                    }
                }
            }

            while v.is_even() {
                v.div2();

                if c.0.is_even() {
                    c.0.div2();
                } else {
                    let carry = c.0.add_with_carry(&Self::MODULUS);
                    c.0.div2();
                    if !Self::MODULUS_HAS_SPARE_BIT && carry {
                        (c.0).0[N - 1] |= 1 << 63;
                    }
                }
            }

            if v < u {
                u.sub_with_borrow(&v);
                b -= &c;
            } else {
                v.sub_with_borrow(&u);
                c -= &b;
            }
        }

        if u == one {
            Some(b)
        } else {
            Some(c)
        }
    }

    /// Construct a field element from an integer in the range
    /// `0..(Self::MODULUS - 1)`. Returns `None` if the integer is outside
    /// this range.
    fn from_bigint(r: BigInt<N>) -> Option<Fp<Self, N>> {
        let mut r = Fp::new_unchecked(r);
        if r.is_zero() {
            Some(r)
        } else if r.is_geq_modulus() {
            None
        } else {
            r *= &Fp::new_unchecked(Self::R2);
            Some(r)
        }
    }

    /// Convert a field element to an integer in the range `0..(Self::MODULUS -
    /// 1)`.
    fn into_bigint(a: Fp<Self, N>) -> BigInt<N> {
        let mut r = (a.0).0;
        // Montgomery Reduction
        for i in 0..N {
            let k = r[i].wrapping_mul(Self::INV);
            let mut carry = 0;

            crate::biginteger::arithmetic::mac_with_carry(
                r[i],
                k,
                Self::MODULUS.0[0],
                &mut carry,
            );
            for j in 1..N {
                r[(j + i) % N] = crate::biginteger::arithmetic::mac_with_carry(
                    r[(j + i) % N],
                    k,
                    Self::MODULUS.0[j],
                    &mut carry,
                );
            }
            r[i % N] = carry;
        }

        BigInt::new(r)
    }
}

/// Compute -M^{-1} mod 2^64.
pub const fn inv<T: FpConfig<N>, const N: usize>() -> u64 {
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
    crate::const_for!((_i in 0..63) {
        // Square
        inv = inv.wrapping_mul(inv);
        // Multiply
        inv = inv.wrapping_mul(T::MODULUS.0[0]);
    });
    inv.wrapping_neg()
}

#[inline]
pub const fn can_use_no_carry_mul_optimization<
    T: FpConfig<N>,
    const N: usize,
>() -> bool {
    // Checking the modulus at compile time
    let mut all_remaining_bits_are_one = T::MODULUS.0[N - 1] == u64::MAX >> 1;
    crate::const_for!((i in 1..N) {
        all_remaining_bits_are_one  &= T::MODULUS.0[N - i - 1] == u64::MAX;
    });
    modulus_has_spare_bit::<T, N>() && !all_remaining_bits_are_one
}

#[inline]
pub const fn modulus_has_spare_bit<T: FpConfig<N>, const N: usize>() -> bool {
    T::MODULUS.0[N - 1] >> 63 == 0
}

/// Represents an element of the prime field F_p, where `p == P::MODULUS`.
/// This type can represent elements in any field of size at most N * 64 bits.
#[derive(Educe)]
#[educe(Default, Hash, Clone, Copy, PartialEq, Eq)]
pub struct Fp<P: FpConfig<N>, const N: usize>(
    /// Contains the element in Montgomery form for efficient multiplication.
    /// To convert an element to a [`BigInt`](struct@BigInt), use `into_bigint`
    /// or `into`.
    #[doc(hidden)]
    pub BigInt<N>,
    #[doc(hidden)] pub PhantomData<P>,
);

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

impl<P: FpConfig<N>, const N: usize> Fp<P, N> {
    #[doc(hidden)]
    pub const INV: u64 = P::INV;
    #[doc(hidden)]
    pub const R: BigInt<N> = P::R;
    #[doc(hidden)]
    pub const R2: BigInt<N> = P::R2;

    /// Construct a new field element from its underlying
    /// [`struct@BigInt`] data type.
    #[inline]
    pub const fn new(element: BigInt<N>) -> Self {
        let mut r = Self(element, PhantomData);
        if r.const_is_zero() {
            r
        } else {
            r = r.mul(&Fp(P::R2, PhantomData));
            r
        }
    }

    /// Construct a new field element from its underlying
    /// [`struct@BigInt`] data type.
    ///
    /// Unlike [`Self::new`], this method does not perform Montgomery reduction.
    /// Thus, this method should be used only when constructing
    /// an element from an integer that has already been put in
    /// Montgomery form.
    #[inline]
    pub const fn new_unchecked(element: BigInt<N>) -> Self {
        Self(element, PhantomData)
    }

    const fn const_is_zero(&self) -> bool {
        self.0.const_is_zero()
    }

    #[doc(hidden)]
    const fn const_neg(self) -> Self {
        if !self.const_is_zero() {
            Self::new_unchecked(Self::sub_with_borrow(&P::MODULUS, &self.0))
        } else {
            self
        }
    }

    /// Interpret a set of limbs (along with a sign) as a field element.
    /// For *internal* use only; please use the `ark_ff::MontFp` macro instead
    /// of this method
    #[doc(hidden)]
    pub const fn from_sign_and_limbs(is_positive: bool, limbs: &[u64]) -> Self {
        let mut repr = BigInt::<N>([0; N]);
        assert!(limbs.len() <= N);
        crate::const_for!((i in 0..(limbs.len())) {
            repr.0[i] = limbs[i];
        });
        let res = Self::new(repr);
        if is_positive {
            res
        } else {
            res.const_neg()
        }
    }

    const fn mul_without_cond_subtract(mut self, other: &Self) -> (bool, Self) {
        let (mut lo, mut hi) = ([0u64; N], [0u64; N]);
        crate::const_for!((i in 0..N) {
            let mut carry = 0;
            crate::const_for!((j in 0..N) {
                let k = i + j;
                if k >= N {
                    hi[k - N] = mac_with_carry!(hi[k - N], (self.0).0[i], (other.0).0[j], &mut carry);
                } else {
                    lo[k] = mac_with_carry!(lo[k], (self.0).0[i], (other.0).0[j], &mut carry);
                }
            });
            hi[i] = carry;
        });
        // Montgomery reduction
        let mut carry2 = 0;
        crate::const_for!((i in 0..N) {
            let tmp = lo[i].wrapping_mul(P::INV);
            let mut carry;
            mac!(lo[i], tmp, P::MODULUS.0[0], &mut carry);
            crate::const_for!((j in 1..N) {
                let k = i + j;
                if k >= N {
                    hi[k - N] = mac_with_carry!(hi[k - N], tmp, P::MODULUS.0[j], &mut carry);
                }  else {
                    lo[k] = mac_with_carry!(lo[k], tmp, P::MODULUS.0[j], &mut carry);
                }
            });
            hi[i] = adc!(hi[i], carry, &mut carry2);
        });

        crate::const_for!((i in 0..N) {
            (self.0).0[i] = hi[i];
        });
        (carry2 != 0, self)
    }

    const fn mul(self, other: &Self) -> Self {
        let (carry, res) = self.mul_without_cond_subtract(other);
        if P::MODULUS_HAS_SPARE_BIT {
            res.const_subtract_modulus()
        } else {
            res.const_subtract_modulus_with_carry(carry)
        }
    }

    const fn const_is_valid(&self) -> bool {
        crate::const_for!((i in 0..N) {
            if (self.0).0[N - i - 1] < P::MODULUS.0[N - i - 1] {
                return true
            } else if (self.0).0[N - i - 1] > P::MODULUS.0[N - i - 1] {
                return false
            }
        });
        false
    }

    #[inline]
    const fn const_subtract_modulus(mut self) -> Self {
        if !self.const_is_valid() {
            self.0 = Self::sub_with_borrow(&self.0, &P::MODULUS);
        }
        self
    }

    #[inline]
    const fn const_subtract_modulus_with_carry(mut self, carry: bool) -> Self {
        if carry || !self.const_is_valid() {
            self.0 = Self::sub_with_borrow(&self.0, &P::MODULUS);
        }
        self
    }

    const fn sub_with_borrow(a: &BigInt<N>, b: &BigInt<N>) -> BigInt<N> {
        a.const_sub_with_borrow(b).0
    }

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
        64 * N - (Self::MODULUS_BIT_SIZE as usize)
    }
}

impl<P: FpConfig<N>, const N: usize> ark_std::fmt::Debug for Fp<P, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> ark_std::fmt::Result {
        ark_std::fmt::Debug::fmt(&self.into_bigint(), f)
    }
}

impl<P: FpConfig<N>, const N: usize> Zero for Fp<P, N> {
    #[inline]
    fn zero() -> Self {
        P::ZERO
    }

    #[inline]
    fn is_zero(&self) -> bool {
        *self == P::ZERO
    }
}

impl<P: FpConfig<N>, const N: usize> One for Fp<P, N> {
    #[inline]
    fn one() -> Self {
        P::ONE
    }

    #[inline]
    fn is_one(&self) -> bool {
        *self == P::ONE
    }
}

impl<P: FpConfig<N>, const N: usize> AdditiveGroup for Fp<P, N> {
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

impl<P: FpConfig<N>, const N: usize> Field for Fp<P, N> {
    const ONE: Self = P::ONE;

    fn extension_degree() -> u64 {
        1
    }

    #[inline]
    fn characteristic() -> &'static [u64] {
        P::MODULUS.as_ref()
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
                crate::const_helpers::SerBuffer::<N>::zeroed();
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
                flag_location.saturating_sub(8 * (N - 1));

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
            Self::deserialize_compressed(&result_bytes.as_slice()[..(N * 8)])
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
}

impl<P: FpConfig<N>, const N: usize> PrimeField for Fp<P, N> {
    type BigInt = BigInt<N>;

    const MODULUS: Self::BigInt = P::MODULUS;
    const MODULUS_BIT_SIZE: u32 = P::MODULUS.const_num_bits();
    const TRACE: Self::BigInt = P::MODULUS.two_adic_coefficient();

    #[inline]
    fn from_bigint(r: BigInt<N>) -> Option<Self> {
        P::from_bigint(r)
    }

    fn into_bigint(self) -> BigInt<N> {
        P::into_bigint(self)
    }
}

/// Note that this implementation of `Ord` compares field elements viewing
/// them as integers in the range 0, 1, ..., P::MODULUS - 1. However, other
/// implementations of `PrimeField` might choose a different ordering, and
/// as such, users should use this `Ord` for applications where
/// any ordering suffices (like in a BTreeMap), and not in applications
/// where a particular ordering is required.
impl<P: FpConfig<N>, const N: usize> Ord for Fp<P, N> {
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
impl<P: FpConfig<N>, const N: usize> PartialOrd for Fp<P, N> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<P: FpConfig<N>, const N: usize> From<u128> for Fp<P, N> {
    fn from(mut other: u128) -> Self {
        let mut result = BigInt::default();
        if N == 1 {
            result.0[0] = (other % u128::from(P::MODULUS.0[0])) as u64;
        } else if N == 2 || P::MODULUS.0[2..].iter().all(|&x| x == 0) {
            let mod_as_u128 =
                P::MODULUS.0[0] as u128 + ((P::MODULUS.0[1] as u128) << 64);
            other %= mod_as_u128;
            result.0[0] = ((other << 64) >> 64) as u64;
            result.0[1] = (other >> 64) as u64;
        } else {
            result.0[0] = ((other << 64) >> 64) as u64;
            result.0[1] = (other >> 64) as u64;
        }
        Self::from_bigint(result).unwrap()
    }
}

impl<P: FpConfig<N>, const N: usize> From<i128> for Fp<P, N> {
    fn from(other: i128) -> Self {
        let abs = Self::from(other.unsigned_abs());
        if other.is_positive() {
            abs
        } else {
            -abs
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<bool> for Fp<P, N> {
    fn from(other: bool) -> Self {
        if N == 1 {
            Self::from_bigint(BigInt::from(u64::from(other) % P::MODULUS.0[0]))
                .unwrap()
        } else {
            Self::from_bigint(BigInt::from(u64::from(other))).unwrap()
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<u64> for Fp<P, N> {
    fn from(other: u64) -> Self {
        if N == 1 {
            Self::from_bigint(BigInt::from(other % P::MODULUS.0[0])).unwrap()
        } else {
            Self::from_bigint(BigInt::from(other)).unwrap()
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<i64> for Fp<P, N> {
    fn from(other: i64) -> Self {
        let abs = Self::from(other.unsigned_abs());
        if other.is_positive() {
            abs
        } else {
            -abs
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<u32> for Fp<P, N> {
    fn from(other: u32) -> Self {
        if N == 1 {
            Self::from_bigint(BigInt::from(u64::from(other) % P::MODULUS.0[0]))
                .unwrap()
        } else {
            Self::from_bigint(BigInt::from(other)).unwrap()
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<i32> for Fp<P, N> {
    fn from(other: i32) -> Self {
        let abs = Self::from(other.unsigned_abs());
        if other.is_positive() {
            abs
        } else {
            -abs
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<u16> for Fp<P, N> {
    fn from(other: u16) -> Self {
        if N == 1 {
            Self::from_bigint(BigInt::from(u64::from(other) % P::MODULUS.0[0]))
                .unwrap()
        } else {
            Self::from_bigint(BigInt::from(other)).unwrap()
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<i16> for Fp<P, N> {
    fn from(other: i16) -> Self {
        let abs = Self::from(other.unsigned_abs());
        if other.is_positive() {
            abs
        } else {
            -abs
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<u8> for Fp<P, N> {
    fn from(other: u8) -> Self {
        if N == 1 {
            Self::from_bigint(BigInt::from(u64::from(other) % P::MODULUS.0[0]))
                .unwrap()
        } else {
            Self::from_bigint(BigInt::from(other)).unwrap()
        }
    }
}

impl<P: FpConfig<N>, const N: usize> From<i8> for Fp<P, N> {
    fn from(other: i8) -> Self {
        let abs = Self::from(other.unsigned_abs());
        if other.is_positive() {
            abs
        } else {
            -abs
        }
    }
}

impl<P: FpConfig<N>, const N: usize>
    ark_std::rand::distributions::Distribution<Fp<P, N>>
    for ark_std::rand::distributions::Standard
{
    #[inline]
    fn sample<R: ark_std::rand::Rng + ?Sized>(&self, rng: &mut R) -> Fp<P, N> {
        loop {
            let mut tmp = Fp(
                rng.sample(ark_std::rand::distributions::Standard),
                PhantomData,
            );
            let shave_bits = Fp::<P, N>::num_bits_to_shave();
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

impl<P: FpConfig<N>, const N: usize> CanonicalSerializeWithFlags for Fp<P, N> {
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
        bytes.copy_from_u64_slice(&self.into_bigint().0);
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

impl<P: FpConfig<N>, const N: usize> CanonicalSerialize for Fp<P, N> {
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

impl<P: FpConfig<N>, const N: usize> CanonicalDeserializeWithFlags
    for Fp<P, N>
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

impl<P: FpConfig<N>, const N: usize> Valid for Fp<P, N> {
    fn check(&self) -> Result<(), SerializationError> {
        Ok(())
    }
}

impl<P: FpConfig<N>, const N: usize> CanonicalDeserialize for Fp<P, N> {
    fn deserialize_with_mode<R: ark_std::io::Read>(
        reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        Self::deserialize_with_flags::<R, EmptyFlags>(reader).map(|(r, _)| r)
    }
}

impl<P: FpConfig<N>, const N: usize> FromStr for Fp<P, N> {
    type Err = ();

    /// Interpret a string of numbers as a (congruent) prime field element.
    /// Does not accept unnecessary leading zeroes or a blank string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use num_bigint::{BigInt, BigUint};
        use num_traits::Signed;

        let modulus = BigInt::from(P::MODULUS);
        let mut a = BigInt::from_str(s).map_err(|_| ())? % &modulus;
        if a.is_negative() {
            a += modulus
        }
        BigUint::try_from(a)
            .map_err(|_| ())
            .and_then(TryFrom::try_from)
            .ok()
            .and_then(Self::from_bigint)
            .ok_or(())
    }
}

/// Outputs a string containing the value of `self`,
/// represented as a decimal without leading zeroes.
impl<P: FpConfig<N>, const N: usize> Display for Fp<P, N> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let string = self.into_bigint().to_string();
        write!(f, "{}", string)
    }
}

impl<P: FpConfig<N>, const N: usize> Neg for Fp<P, N> {
    type Output = Self;

    #[inline]
    #[must_use]
    fn neg(mut self) -> Self {
        P::neg_in_place(&mut self);
        self
    }
}

impl<'a, P: FpConfig<N>, const N: usize> Add<&'a Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &Self) -> Self {
        self.add_assign(other);
        self
    }
}

impl<'a, P: FpConfig<N>, const N: usize> Sub<&'a Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &Self) -> Self {
        self.sub_assign(other);
        self
    }
}

impl<'a, P: FpConfig<N>, const N: usize> Mul<&'a Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: &Self) -> Self {
        self.mul_assign(other);
        self
    }
}

impl<'a, P: FpConfig<N>, const N: usize> Div<&'a Fp<P, N>> for Fp<P, N> {
    type Output = Self;

    /// Returns `self * other.inverse()` if `other.inverse()` is `Some`, and
    /// panics otherwise.
    #[inline]
    fn div(mut self, other: &Self) -> Self {
        self.mul_assign(&other.inverse().unwrap());
        self
    }
}

impl<'a, 'b, P: FpConfig<N>, const N: usize> Add<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn add(self, other: &'b Fp<P, N>) -> Fp<P, N> {
        let mut result = *self;
        result.add_assign(other);
        result
    }
}

impl<'a, 'b, P: FpConfig<N>, const N: usize> Sub<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn sub(self, other: &Fp<P, N>) -> Fp<P, N> {
        let mut result = *self;
        result.sub_assign(other);
        result
    }
}

impl<'a, 'b, P: FpConfig<N>, const N: usize> Mul<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn mul(self, other: &Fp<P, N>) -> Fp<P, N> {
        let mut result = *self;
        result.mul_assign(other);
        result
    }
}

impl<'a, 'b, P: FpConfig<N>, const N: usize> Div<&'b Fp<P, N>>
    for &'a Fp<P, N>
{
    type Output = Fp<P, N>;

    #[inline]
    fn div(self, other: &Fp<P, N>) -> Fp<P, N> {
        let mut result = *self;
        result.div_assign(other);
        result
    }
}

impl<'a, P: FpConfig<N>, const N: usize> AddAssign<&'a Self> for Fp<P, N> {
    #[inline]
    fn add_assign(&mut self, other: &Self) {
        P::add_assign(self, other)
    }
}

impl<'a, P: FpConfig<N>, const N: usize> SubAssign<&'a Self> for Fp<P, N> {
    #[inline]
    fn sub_assign(&mut self, other: &Self) {
        P::sub_assign(self, other);
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::Add<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn add(mut self, other: Self) -> Self {
        self.add_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::Add<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn add(mut self, other: &'a mut Self) -> Self {
        self.add_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::Sub<Self> for Fp<P, N> {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: Self) -> Self {
        self.sub_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::Sub<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &'a mut Self) -> Self {
        self.sub_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::iter::Sum<Self> for Fp<P, N> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::iter::Sum<&'a Self>
    for Fp<P, N>
{
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), core::ops::Add::add)
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::AddAssign<Self> for Fp<P, N> {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::SubAssign<Self> for Fp<P, N> {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::AddAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn add_assign(&mut self, other: &'a mut Self) {
        self.add_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::SubAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn sub_assign(&mut self, other: &'a mut Self) {
        self.sub_assign(&*other)
    }
}

impl<'a, P: FpConfig<N>, const N: usize> MulAssign<&'a Self> for Fp<P, N> {
    fn mul_assign(&mut self, other: &Self) {
        P::mul_assign(self, other)
    }
}

/// Computes `self *= other.inverse()` if `other.inverse()` is `Some`, and
/// panics otherwise.
impl<'a, P: FpConfig<N>, const N: usize> DivAssign<&'a Self> for Fp<P, N> {
    #[inline(always)]
    fn div_assign(&mut self, other: &Self) {
        self.mul_assign(&other.inverse().unwrap());
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::Mul<Self> for Fp<P, N> {
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: Self) -> Self {
        self.mul_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::Div<Self> for Fp<P, N> {
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: Self) -> Self {
        self.div_assign(&other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::Mul<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline(always)]
    fn mul(mut self, other: &'a mut Self) -> Self {
        self.mul_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::Div<&'a mut Self>
    for Fp<P, N>
{
    type Output = Self;

    #[inline(always)]
    fn div(mut self, other: &'a mut Self) -> Self {
        self.div_assign(&*other);
        self
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::iter::Product<Self> for Fp<P, N> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), core::ops::Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::iter::Product<&'a Self>
    for Fp<P, N>
{
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::one(), Mul::mul)
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::MulAssign<Self> for Fp<P, N> {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        self.mul_assign(&other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::DivAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn div_assign(&mut self, other: &'a mut Self) {
        self.div_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl<'a, P: FpConfig<N>, const N: usize> core::ops::MulAssign<&'a mut Self>
    for Fp<P, N>
{
    #[inline(always)]
    fn mul_assign(&mut self, other: &'a mut Self) {
        self.mul_assign(&*other)
    }
}

#[allow(unused_qualifications)]
impl<P: FpConfig<N>, const N: usize> core::ops::DivAssign<Self> for Fp<P, N> {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        self.div_assign(&other)
    }
}

impl<P: FpConfig<N>, const N: usize> zeroize::Zeroize for Fp<P, N> {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl<P: FpConfig<N>, const N: usize> From<num_bigint::BigUint> for Fp<P, N> {
    #[inline]
    fn from(val: num_bigint::BigUint) -> Fp<P, N> {
        Fp::<P, N>::from_le_bytes_mod_order(&val.to_bytes_le())
    }
}

impl<P: FpConfig<N>, const N: usize> From<Fp<P, N>> for num_bigint::BigUint {
    #[inline(always)]
    fn from(other: Fp<P, N>) -> Self {
        other.into_bigint().into()
    }
}

impl<P: FpConfig<N>, const N: usize> From<Fp<P, N>> for BigInt<N> {
    #[inline(always)]
    fn from(fp: Fp<P, N>) -> Self {
        fp.into_bigint()
    }
}

impl<P: FpConfig<N>, const N: usize> From<BigInt<N>> for Fp<P, N> {
    /// Converts `Self::BigInteger` into `Self`
    #[inline(always)]
    fn from(int: BigInt<N>) -> Self {
        Self::from_bigint(int).unwrap()
    }
}
