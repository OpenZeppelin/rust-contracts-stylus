//! This module contains low-level arithmetic functions for
//! big integer's limbs.

// Actually cast truncations are a part of the logic here.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]

use num_traits::ConstOne;

/// A single limb of a big integer represented by 64-bits.
pub type Limb = u64;

/// Array of [`Limb`]s.
pub type Limbs<const N: usize> = [Limb; N];

/// A wide limb represented by 128-bits.
///
/// Twice larger than [`Limb`].
pub type WideLimb = u128;

/// Multiply two [`Limb`]'s and return widened result.
#[inline(always)]
#[must_use]
pub const fn widening_mul(a: Limb, b: Limb) -> WideLimb {
    #[cfg(not(target_family = "wasm"))]
    {
        a as WideLimb * b as WideLimb
    }
    #[cfg(target_family = "wasm")]
    {
        widening_mul_wasm(a, b)
    }
}

/// Multiply two [`Limb`]'s and return widened result.
///
/// This function is optimized for wasm target, due to inefficiency of
/// 128-bit multiplication in WebAssembly.
#[inline(always)]
#[doc(hidden)]
#[allow(dead_code)]
const fn widening_mul_wasm(a: Limb, b: Limb) -> WideLimb {
    let a_lo = a as u32 as Limb;
    let a_hi = a >> 32;
    let b_lo = b as u32 as Limb;
    let b_hi = b >> 32;

    let lolo = (a_lo * b_lo) as WideLimb;
    let lohi = ((a_lo * b_hi) as WideLimb) << 32;
    let hilo = ((a_hi * b_lo) as WideLimb) << 32;
    let hihi = ((a_hi * b_hi) as WideLimb) << 64;
    (lolo | hihi) + (lohi + hilo)
}

/// Calculate `a + b * c`, returning the lower 64 bits of the result and setting
/// `carry` to the upper 64 bits.
#[inline(always)]
#[must_use]
pub const fn mac(a: Limb, b: Limb, c: Limb) -> (Limb, Limb) {
    let a = a as WideLimb;
    let tmp = a + widening_mul(b, c);
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Calculate `a + (b * c) + carry`, returning the least significant digit
/// and setting carry to the most significant digit.
#[inline(always)]
#[must_use]
pub const fn carrying_mac(
    a: Limb,
    b: Limb,
    c: Limb,
    carry: Limb,
) -> (Limb, Limb) {
    let a = a as WideLimb;
    let carry = carry as WideLimb;
    let tmp = a + widening_mul(b, c) + carry;
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Calculate `a = a + b + carry` and return the result and carry.
#[inline(always)]
#[must_use]
pub const fn adc(a: Limb, b: Limb, carry: bool) -> (Limb, bool) {
    let a = a as WideLimb;
    let b = b as WideLimb;
    let carry = carry as WideLimb;
    let tmp = a + b + carry;
    let carry = (tmp >> Limb::BITS) != 0;
    (tmp as Limb, carry)
}

/// Sets a = a + b + carry, and returns the new carry.
#[inline(always)]
pub fn adc_assign(a: &mut Limb, b: Limb, carry: bool) -> bool {
    let tmp = *a as WideLimb + b as WideLimb + carry as WideLimb;
    *a = tmp as Limb;
    let carry = tmp >> Limb::BITS;
    carry != 0
}

/// Calculate `a = a - b - borrow` and return the result and borrow.
#[inline(always)]
#[must_use]
pub const fn sbb(a: Limb, b: Limb, borrow: bool) -> (Limb, bool) {
    let a = a as WideLimb;
    let b = b as WideLimb;
    let borrow = borrow as WideLimb;
    // Protects from overflow, when `a < b + borrow`.
    let overflow_protection = WideLimb::ONE << Limb::BITS;
    let tmp = overflow_protection + a - b - borrow;
    let borrow = tmp >> Limb::BITS == 0;
    // overflow_protection will be truncated on cast.
    (tmp as Limb, borrow)
}

/// Sets a = a - b - borrow, and returns the borrow.
#[inline(always)]
pub fn sbb_assign(a: &mut Limb, b: Limb, borrow: bool) -> bool {
    let (sub, borrow1) = a.overflowing_sub(b);
    let (sub, borrow2) = sub.overflowing_sub(borrow as Limb);
    *a = sub;
    borrow1 | borrow2
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn check_widening_mul() {
        proptest!(|(a: Limb, b: Limb)|{
            let std_mul_result = widening_mul(a, b);
            let wasm_mul_result = widening_mul_wasm(a, b);
            prop_assert_eq!(std_mul_result, wasm_mul_result);
        });
    }
}
