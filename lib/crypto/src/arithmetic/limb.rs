// TODO#q: we need carrying_mac and mac
// TODO#q: Rename u64 and u128 to Limb and WideLimb
// TODO#q: Rename functions *_with_carry to carrying_*

pub type Limb = u64;
pub type Limbs<const N: usize> = [Limb; N];
pub type WideLimb = u128;

#[inline(always)]
#[doc(hidden)]
pub const fn widening_mul(a: u64, b: u64) -> u128 {
    #[cfg(not(target_family = "wasm"))]
    {
        a as u128 * b as u128
    }
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

/// Calculate a + b * c, returning the lower 64 bits of the result and setting
/// `carry` to the upper 64 bits.
#[inline(always)]
#[doc(hidden)]
pub const fn mac(a: u64, b: u64, c: u64) -> (u64, u64) {
    let tmp = (a as u128) + widening_mul(b, c);
    let carry = (tmp >> 64) as u64;
    (tmp as u64, carry)
}

/// Calculate a + (b * c) + carry, returning the least significant digit
/// and setting carry to the most significant digit.
#[inline(always)]
#[doc(hidden)]
pub const fn carrying_mac(a: u64, b: u64, c: u64, carry: u64) -> (u64, u64) {
    let tmp = (a as u128) + widening_mul(b, c) + (carry as u128);
    let carry = (tmp >> 64) as u64;
    (tmp as u64, carry)
}

pub const fn ct_mac_with_carry(
    a: Limb,
    b: Limb,
    c: Limb,
    carry: Limb,
) -> (Limb, Limb) {
    let a = a as WideLimb;
    let b = b as WideLimb;
    let c = c as WideLimb;
    let carry = carry as WideLimb;
    let ret = a + (b * c) + carry;
    (ret as Limb, (ret >> Limb::BITS) as Limb)
}

/// Calculate a + b * c, discarding the lower 64 bits of the result and setting
/// `carry` to the upper 64 bits.
#[inline(always)]
#[doc(hidden)]
pub fn mac_discard(a: u64, b: u64, c: u64, carry: &mut u64) {
    let tmp = (a as u128) + widening_mul(b, c);
    *carry = (tmp >> 64) as u64;
}

// TODO#q: adc can be unified with adc_for_add_with_carry
/// Calculate `a = a + b + carry` and return the result and carry.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let tmp = a as u128 + b as u128 + carry as u128;
    let carry = (tmp >> 64) as u64;
    (tmp as u64, carry)
}

/// Sets a = a + b + carry, and returns the new carry.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub fn adc_for_add_with_carry(a: &mut u64, b: u64, carry: bool) -> bool {
    let (sum, carry1) = a.overflowing_add(b);
    let (sum, carry2) = sum.overflowing_add(carry as u64);
    *a = sum;
    carry1 | carry2
}

// TODO#q: sbb can be unified with sbb_for_sub_with_borrow
/// Calculate `a = a - b - borrow` and return the result and borrow.
pub const fn sbb(a: u64, b: u64, borrow: u64) -> (u64, u64) {
    let tmp = (1u128 << 64) + (a as u128) - (b as u128) - (borrow as u128);
    let borrow = if tmp >> 64 == 0 { 1 } else { 0 };
    (tmp as u64, borrow)
}

/// Sets a = a - b - borrow, and returns the borrow.
#[inline(always)]
#[allow(unused_mut)]
#[doc(hidden)]
pub fn sbb_for_sub_with_borrow(a: &mut u64, b: u64, borrow: bool) -> bool {
    let (sub, borrow1) = a.overflowing_sub(b);
    let (sub, borrow2) = sub.overflowing_sub(borrow as u64);
    *a = sub;
    borrow1 | borrow2
}

/// Computes `lhs * rhs`, returning the low and the high limbs of the result.
#[inline(always)]
pub const fn ct_mul_wide(lhs: Limb, rhs: Limb) -> (Limb, Limb) {
    let a = lhs as WideLimb;
    let b = rhs as WideLimb;
    let ret = a * b;
    (ret as Limb, (ret >> Limb::BITS) as Limb)
}

// TODO#q: merge with adc function
/// Computes `lhs + rhs + carry`, returning the result along with the new carry
/// (0, 1, or 2).
// NOTE#q: crypto_bigint
#[inline(always)]
pub const fn ct_adc(lhs: Limb, rhs: Limb, carry: Limb) -> (Limb, Limb) {
    // We could use `Word::overflowing_add()` here analogous to
    // `overflowing_add()`, but this version seems to produce a slightly
    // better assembly.
    let a = lhs as WideLimb;
    let b = rhs as WideLimb;
    let carry = carry as WideLimb;
    let ret = a + b + carry;
    (ret as Limb, (ret >> Limb::BITS) as Limb)
}

// TODO#q: add unit tests for limb.rs
