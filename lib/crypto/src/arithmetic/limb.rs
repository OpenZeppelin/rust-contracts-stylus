use num_traits::{ConstOne, ConstZero};

pub type Limb = u64;
pub type Limbs<const N: usize> = [Limb; N];
pub type WideLimb = u128;

/// Multiply two [`Limb`]'s and return widened result.
#[inline(always)]
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
pub const fn mac(a: Limb, b: Limb, c: Limb) -> (Limb, Limb) {
    let a = a as WideLimb;
    let tmp = a + widening_mul(b, c);
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Calculate `a + (b * c) + carry`, returning the least significant digit
/// and setting carry to the most significant digit.
#[inline(always)]
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
pub const fn adc(a: Limb, b: Limb, carry: Limb) -> (Limb, Limb) {
    let a = a as WideLimb;
    let b = b as WideLimb;
    let carry = carry as WideLimb;
    let tmp = a + b + carry;
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Sets a = a + b + carry, and returns the new carry.
#[inline(always)]
pub fn adc_assign(a: &mut Limb, b: Limb, carry: bool) -> bool {
    let (sum, carry1) = a.overflowing_add(b);
    let (sum, carry2) = sum.overflowing_add(carry as Limb);
    *a = sum;
    carry1 | carry2
}

/// Calculate `a = a - b - borrow` and return the result and borrow.
#[inline(always)]
pub const fn sbb(a: Limb, b: Limb, borrow: Limb) -> (Limb, Limb) {
    let a = a as WideLimb;
    let b = b as WideLimb;
    let borrow = borrow as WideLimb;
    let tmp = (WideLimb::ONE << Limb::BITS) + a - b - borrow;
    let borrow = if tmp >> Limb::BITS == 0 { Limb::ONE } else { Limb::ZERO };
    (tmp as Limb, borrow)
}

/// Sets a = a - b - borrow, and returns the borrow.
#[inline(always)]
#[allow(unused_mut)]
pub fn sbb_assign(a: &mut Limb, b: Limb, borrow: bool) -> bool {
    let (sub, borrow1) = a.overflowing_sub(b);
    let (sub, borrow2) = sub.overflowing_sub(borrow as Limb);
    *a = sub;
    borrow1 | borrow2
}

#[cfg(all(test, feature = "std"))]
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
