// TODO#q: we need carrying_mac and mac
// TODO#q: Rename u64 and u128 to Limb and WideLimb
// TODO#q: Rename functions *_with_carry to carrying_*

use num_traits::ConstOne;

pub type Limb = u64;
pub type Limbs<const N: usize> = [Limb; N];
pub type WideLimb = u128;

#[inline(always)]
#[doc(hidden)]
pub const fn widening_mul(a: Limb, b: Limb) -> WideLimb {
    #[cfg(not(target_family = "wasm"))]
    {
        a as WideLimb * b as WideLimb
    }
    // TODO#q: check widening_mul for wasm in unit tests
    #[cfg(target_family = "wasm")]
    {
        let a_lo = a as u32 as u64;
        let a_hi = a >> 32;
        let b_lo = b as u32 as u64;
        let b_hi = b >> 32;

        let lolo = (a_lo * b_lo) as u128;
        let lohi = ((a_lo * b_hi) as u128) << 32;
        let hilo = ((a_hi * b_lo) as u128) << 32;
        let hihi = ((a_hi * b_hi) as u128) << 64;
        (lolo | hihi) + (lohi + hilo)
    }
}

/// Calculate `a + b * c`, returning the lower 64 bits of the result and setting
/// `carry` to the upper 64 bits.
#[inline(always)]
#[doc(hidden)]
pub const fn mac(a: Limb, b: Limb, c: Limb) -> (Limb, Limb) {
    let tmp = (a as WideLimb) + widening_mul(b, c);
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Calculate `a + (b * c) + carry`, returning the least significant digit
/// and setting carry to the most significant digit.
#[inline(always)]
#[doc(hidden)]
pub const fn carrying_mac(
    a: Limb,
    b: Limb,
    c: Limb,
    carry: Limb,
) -> (Limb, Limb) {
    let tmp = (a as WideLimb) + widening_mul(b, c) + (carry as WideLimb);
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Calculate `a = a + b + carry` and return the result and carry.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub const fn adc(a: Limb, b: Limb, carry: Limb) -> (Limb, Limb) {
    let tmp = a as WideLimb + b as WideLimb + carry as WideLimb;
    let carry = (tmp >> Limb::BITS) as Limb;
    (tmp as Limb, carry)
}

/// Sets a = a + b + carry, and returns the new carry.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub fn adc_assign(a: &mut Limb, b: Limb, carry: bool) -> bool {
    let (sum, carry1) = a.overflowing_add(b);
    let (sum, carry2) = sum.overflowing_add(carry as Limb);
    *a = sum;
    carry1 | carry2
}

/// Calculate `a = a - b - borrow` and return the result and borrow.
pub const fn sbb(a: Limb, b: Limb, borrow: Limb) -> (Limb, Limb) {
    let tmp = (WideLimb::ONE << Limb::BITS) + (a as WideLimb)
        - (b as WideLimb)
        - (borrow as WideLimb);
    let borrow = if tmp >> Limb::BITS == 0 { 1 } else { 0 };
    (tmp as Limb, borrow)
}

/// Sets a = a - b - borrow, and returns the borrow.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub fn sbb_assign(a: &mut Limb, b: Limb, borrow: bool) -> bool {
    let (sub, borrow1) = a.overflowing_sub(b);
    let (sub, borrow2) = sub.overflowing_sub(borrow as Limb);
    *a = sub;
    borrow1 | borrow2
}

// TODO#q: add unit tests for limb.rs
