// TODO#q: add constant function for ruint

use ruint::Uint;

pub const fn new<const BITS: usize, const LIMBS: usize>(
    value: [u64; LIMBS],
) -> Uint<BITS, LIMBS> {
    todo!()
}

pub const fn zero<const BITS: usize, const LIMBS: usize>() -> Uint<BITS, LIMBS>
{
    Uint::<BITS, LIMBS>::ZERO
}

/// Return the value 1.
pub const fn one<const BITS: usize, const LIMBS: usize>() -> Uint<BITS, LIMBS> {
    let mut limbs = [0u64; LIMBS];
    limbs[0] = 1;
    Uint::from_limbs(limbs)
}

/// Check if `num` is even.
#[doc(hidden)]
pub const fn const_is_even<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> bool {
    num.as_limbs()[0] % 2 == 0
}

/// Check if `num` is odd.
pub const fn const_is_odd<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> bool {
    num.as_limbs()[0] % 2 == 0
}

/// Compute the value of `num % 4`.
pub const fn mod_4<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> u8 {
    // We only need the two last bits for the modulo 4 operation.
    let first_limb = num.as_limbs()[0] % 4;
    let two_last_bits = (first_limb << 62) >> 62;
    (two_last_bits % 4) as u8
}

/// Compute a right shift of `Uint<BITS, LIMBS>`
/// This is equivalent to a (saturating) division by 2.
#[doc(hidden)]
pub const fn const_shr<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> Uint<BITS, LIMBS> {
    // let mut result = *Uint<BITS, LIMBS>;
    // let mut t = 0;
    // crate::const_for!((i in 0..N) {
    //     let a = result.0[N - i - 1];
    //     let t2 = a << 63;
    //     result.0[N - i - 1] >>= 1;
    //     result.0[N - i - 1] |= t;
    //     t = t2;
    // });
    // result
    todo!()
}

const fn const_geq<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
    other: &Uint<BITS, LIMBS>,
) -> bool {
    // const_for!((i in 0..N) {
    //     let a = Uint<BITS, LIMBS>.0[N - i - 1];
    //     let b = other.0[N - i - 1];
    //     if a < b {
    //         return false;
    //     } else if a > b {
    //         return true;
    //     }
    // });
    // true
    todo!()
}

/// Compute the largest integer `s` such that `Uint<BITS, LIMBS> = 2**s * t + 1`
/// for odd `t`.
#[doc(hidden)]
pub const fn two_adic_valuation<const BITS: usize, const LIMBS: usize>(
    mut num: Uint<BITS, LIMBS>,
) -> u32 {
    // assert!(Uint<BITS, LIMBS>.const_is_odd());
    // let mut two_adicity = 0;
    // // Since `Uint<BITS, LIMBS>` is odd, we can always subtract one
    // // without a borrow
    // Uint<BITS, LIMBS>.0[0] -= 1;
    // while Uint<BITS, LIMBS>.const_is_even() {
    //     Uint<BITS, LIMBS> = Uint<BITS, LIMBS>.const_shr();
    //     two_adicity += 1;
    // }
    // two_adicity
    todo!()
}

/// Compute the smallest odd integer `t` such that `Uint<BITS, LIMBS> = 2**s * t
/// + 1` for some integer `s = Uint<BITS, LIMBS>.two_adic_valuation()`.
#[doc(hidden)]
pub const fn two_adic_coefficient<const BITS: usize, const LIMBS: usize>(
    mut num: Uint<BITS, LIMBS>,
) -> Uint<BITS, LIMBS> {
    // assert!(Uint<BITS, LIMBS>.const_is_odd());
    // // Since `Uint<BITS, LIMBS>` is odd, we can always subtract one
    // // without a borrow
    // Uint<BITS, LIMBS>.0[0] -= 1;
    // while Uint<BITS, LIMBS>.const_is_even() {
    //     Uint<BITS, LIMBS> = Uint<BITS, LIMBS>.const_shr();
    // }
    // assert!(Uint<BITS, LIMBS>.const_is_odd());
    // Uint<BITS, LIMBS>
    todo!()
}

/// Divide `Uint<BITS, LIMBS>` by 2, rounding down if necessary.
/// That is, if `Uint<BITS, LIMBS>.is_odd()`, compute `(Uint<BITS, LIMBS> -
/// 1)/2`. Else, compute `Uint<BITS, LIMBS>/2`.
#[doc(hidden)]
pub const fn divide_by_2_round_down<const BITS: usize, const LIMBS: usize>(
    mut num: Uint<BITS, LIMBS>,
) -> Uint<BITS, LIMBS> {
    // if Uint<BITS, LIMBS>.const_is_odd() {
    //     Uint<BITS, LIMBS>.0[0] -= 1;
    // }
    // Uint<BITS, LIMBS>.const_shr()
    todo!()
}

/// Find the number of bits in the binary decomposition of `Uint<BITS, LIMBS>`.
#[doc(hidden)]
pub const fn const_num_bits<const BITS: usize, const LIMBS: usize>(
    num: Uint<BITS, LIMBS>,
) -> u32 {
    // ((N - 1) * 64) as u32 + (64 - Uint<BITS, LIMBS>.0[N - 1].leading_zeros())
    todo!()
}

#[inline]
pub(crate) const fn const_sub_with_borrow<
    const BITS: usize,
    const LIMBS: usize,
>(
    mut num: Uint<BITS, LIMBS>,
    other: &Uint<BITS, LIMBS>,
) -> (Uint<BITS, LIMBS>, bool) {
    // let mut borrow = 0;
    //
    // const_for!((i in 0..N) {
    //     Uint<BITS, LIMBS>.0[i] = sbb!(Uint<BITS, LIMBS>.0[i], other.0[i],
    // &mut borrow); });
    //
    // (Uint<BITS, LIMBS>, borrow != 0)
    todo!()
}

#[inline]
pub(crate) const fn const_add_with_carry<
    const BITS: usize,
    const LIMBS: usize,
>(
    mut num: Uint<BITS, LIMBS>,
    other: &Uint<BITS, LIMBS>,
) -> (Uint<BITS, LIMBS>, bool) {
    // let mut carry = 0;
    //
    // crate::const_for!((i in 0..N) {
    //     Uint<BITS, LIMBS>.0[i] = adc!(Uint<BITS, LIMBS>.0[i], other.0[i],
    // &mut carry); });
    //
    // (Uint<BITS, LIMBS>, carry != 0)
    todo!()
}

const fn const_mul2_with_carry<const BITS: usize, const LIMBS: usize>(
    mut num: Uint<BITS, LIMBS>,
) -> (Uint<BITS, LIMBS>, bool) {
    // let mut last = 0;
    // crate::const_for!((i in 0..N) {
    //     let a = Uint<BITS, LIMBS>.0[i];
    //     let tmp = a >> 63;
    //     Uint<BITS, LIMBS>.0[i] <<= 1;
    //     Uint<BITS, LIMBS>.0[i] |= last;
    //     last = tmp;
    // });
    // (Uint<BITS, LIMBS>, last != 0)
    todo!()
}

pub(crate) const fn const_is_zero<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> bool {
    // let mut is_zero = true;
    // crate::const_for!((i in 0..N) {
    //     is_zero &= Uint<BITS, LIMBS>.0[i] == 0;
    // });
    // is_zero
    todo!()
}

/// Computes the Montgomery R constant modulo `Uint<BITS, LIMBS>`.
#[doc(hidden)]
pub const fn montgomery_r<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> Uint<BITS, LIMBS> {
    // let two_pow_n_times_64 = crate::const_helpers::RBuffer::<N>([0u64; N],
    // 1); const_modulo!(two_pow_n_times_64, Uint<BITS, LIMBS>)
    todo!()
}

/// Computes the Montgomery R2 constant modulo `Uint<BITS, LIMBS>`.
#[doc(hidden)]
pub const fn montgomery_r2<const BITS: usize, const LIMBS: usize>(
    num: &Uint<BITS, LIMBS>,
) -> Uint<BITS, LIMBS> {
    // let two_pow_n_times_64_square =
    //     crate::const_helpers::R2Buffer::<N>([0u64; N], [0u64; N], 1);
    // const_modulo!(two_pow_n_times_64_square, Uint<BITS, LIMBS>)
    todo!()
}
