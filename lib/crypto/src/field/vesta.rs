use core::convert::TryInto;

use crate::field::fp::{Fp256, MontBackend, MontConfig};

/*
#[derive(MontConfig)]
#[modulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[generator = "5"]
pub struct FqConfig;
*/

pub struct FqConfig;
pub type FpVesta = Fp256<MontBackend<FqConfig, 4>>;

fn fqconfig___() {
    use crate::{
        biginteger::arithmetic as fa,
        field::{fp::Fp, AdditiveGroup},
        BigInt, BigInteger,
    };
    type B = BigInt<4usize>;
    type F = Fp<MontBackend<FqConfig, 4usize>, 4usize>;
    #[automatically_derived]
    impl MontConfig<4usize> for FqConfig {
        const GENERATOR: F = {
            let (is_positive, limbs) = (
                true,
                [
                    12037607305579515999u64,
                    11221139188353527881u64,
                    11411081306099606126u64,
                    3307517586042601304u64,
                ],
            );
            Fp::from_sign_and_limbs(is_positive, &limbs)
        };
        const MODULUS: B = BigInt([
            10108024940646105089u64,
            2469829653919213789u64,
            0u64,
            4611686018427387904u64,
        ]);
        const TWO_ADIC_ROOT_OF_UNITY: F = {
            let (is_positive, limbs) = (
                true,
                [
                    12037607305579515999u64,
                    11221139188353527881u64,
                    11411081306099606126u64,
                    3307517586042601304u64,
                ],
            );
            Fp::from_sign_and_limbs(is_positive, &limbs)
        };

        #[inline(always)]
        fn add_assign(a: &mut F, b: &F) {
            __add_with_carry(&mut a.0, &b.0);
            __subtract_modulus(a);
        }

        #[inline(always)]
        fn sub_assign(a: &mut F, b: &F) {
            if b.0 > a.0 {
                __add_with_carry(
                    &mut a.0,
                    &BigInt([
                        10108024940646105089u64,
                        2469829653919213789u64,
                        0u64,
                        4611686018427387904u64,
                    ]),
                );
            }
            __sub_with_borrow(&mut a.0, &b.0);
        }

        #[inline(always)]
        fn double_in_place(a: &mut F) {
            a.0.mul2();
            __subtract_modulus(a);
        }

        #[doc = r" Sets `a = -a`."]
        #[inline(always)]
        fn neg_in_place(a: &mut F) {
            if *a != F::ZERO {
                let mut tmp = BigInt([
                    10108024940646105089u64,
                    2469829653919213789u64,
                    0u64,
                    4611686018427387904u64,
                ]);
                __sub_with_borrow(&mut tmp, &a.0);
                a.0 = tmp;
            }
        }

        #[inline(always)]
        fn mul_assign(a: &mut F, b: &F) {
            {
                if cfg!(all(
                    feature = "asm",
                    target_feature = "bmi2",
                    target_feature = "adx",
                    target_arch = "x86_64"
                )) {
                    #[cfg(all(
                        feature = "asm",
                        target_feature = "bmi2",
                        target_feature = "adx",
                        target_arch = "x86_64"
                    ))]
                    #[allow(unsafe_code, unused_mut)]
                    ark_ff::x86_64_asm_mul!(4usize, (a.0).0, (b.0).0);
                } else {
                    #[cfg(not(all(
                        feature = "asm",
                        target_feature = "bmi2",
                        target_feature = "adx",
                        target_arch = "x86_64"
                    )))]
                    {
                        let mut r = [0u64; 4usize];
                        let mut carry1 = 0u64;
                        r[0] = fa::mac(
                            r[0],
                            (a.0).0[0],
                            (b.0).0[0usize],
                            &mut carry1,
                        );
                        let k = r[0].wrapping_mul(Self::INV);
                        let mut carry2 = 0u64;
                        fa::mac_discard(
                            r[0],
                            k,
                            10108024940646105089u64,
                            &mut carry2,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[1usize],
                            (a.0).0[1usize],
                            (b.0).0[0usize],
                            &mut carry1,
                        );
                        r[0usize] = fa::mac_with_carry(
                            r[1usize],
                            k,
                            2469829653919213789u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[0usize],
                            &mut carry1,
                        );
                        r[1usize] =
                            fa::mac_with_carry(r[2usize], k, 0u64, &mut carry2);
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[0usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            4611686018427387904u64,
                            &mut carry2,
                        );
                        r[4usize - 1] = carry1 + carry2;
                        let mut carry1 = 0u64;
                        r[0] = fa::mac(
                            r[0],
                            (a.0).0[0],
                            (b.0).0[1usize],
                            &mut carry1,
                        );
                        let k = r[0].wrapping_mul(Self::INV);
                        let mut carry2 = 0u64;
                        fa::mac_discard(
                            r[0],
                            k,
                            10108024940646105089u64,
                            &mut carry2,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[1usize],
                            (a.0).0[1usize],
                            (b.0).0[1usize],
                            &mut carry1,
                        );
                        r[0usize] = fa::mac_with_carry(
                            r[1usize],
                            k,
                            2469829653919213789u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[1usize],
                            &mut carry1,
                        );
                        r[1usize] =
                            fa::mac_with_carry(r[2usize], k, 0u64, &mut carry2);
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[1usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            4611686018427387904u64,
                            &mut carry2,
                        );
                        r[4usize - 1] = carry1 + carry2;
                        let mut carry1 = 0u64;
                        r[0] = fa::mac(
                            r[0],
                            (a.0).0[0],
                            (b.0).0[2usize],
                            &mut carry1,
                        );
                        let k = r[0].wrapping_mul(Self::INV);
                        let mut carry2 = 0u64;
                        fa::mac_discard(
                            r[0],
                            k,
                            10108024940646105089u64,
                            &mut carry2,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[1usize],
                            (a.0).0[1usize],
                            (b.0).0[2usize],
                            &mut carry1,
                        );
                        r[0usize] = fa::mac_with_carry(
                            r[1usize],
                            k,
                            2469829653919213789u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[2usize],
                            &mut carry1,
                        );
                        r[1usize] =
                            fa::mac_with_carry(r[2usize], k, 0u64, &mut carry2);
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[2usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            4611686018427387904u64,
                            &mut carry2,
                        );
                        r[4usize - 1] = carry1 + carry2;
                        let mut carry1 = 0u64;
                        r[0] = fa::mac(
                            r[0],
                            (a.0).0[0],
                            (b.0).0[3usize],
                            &mut carry1,
                        );
                        let k = r[0].wrapping_mul(Self::INV);
                        let mut carry2 = 0u64;
                        fa::mac_discard(
                            r[0],
                            k,
                            10108024940646105089u64,
                            &mut carry2,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[1usize],
                            (a.0).0[1usize],
                            (b.0).0[3usize],
                            &mut carry1,
                        );
                        r[0usize] = fa::mac_with_carry(
                            r[1usize],
                            k,
                            2469829653919213789u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[3usize],
                            &mut carry1,
                        );
                        r[1usize] =
                            fa::mac_with_carry(r[2usize], k, 0u64, &mut carry2);
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[3usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            4611686018427387904u64,
                            &mut carry2,
                        );
                        r[4usize - 1] = carry1 + carry2;
                        (a.0).0 = r;
                    }
                }
            }
            __subtract_modulus(a);
        }

        #[inline(always)]
        fn square_in_place(a: &mut F) {
            {
                if cfg!(all(
                    feature = "asm",
                    target_feature = "bmi2",
                    target_feature = "adx",
                    target_arch = "x86_64"
                )) {
                    #[cfg(all(
                        feature = "asm",
                        target_feature = "bmi2",
                        target_feature = "adx",
                        target_arch = "x86_64"
                    ))]
                    #[allow(unsafe_code, unused_mut)]
                    {
                        ark_ff::x86_64_asm_square!(4usize, (a.0).0);
                    }
                } else {
                    #[cfg(not(all(
                        feature = "asm",
                        target_feature = "bmi2",
                        target_feature = "adx",
                        target_arch = "x86_64"
                    )))]
                    {
                        let mut r = [0u64; 8usize];
                        let mut carry = 0;
                        r[1usize] = fa::mac_with_carry(
                            r[1usize],
                            (a.0).0[0usize],
                            (a.0).0[1usize],
                            &mut carry,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[0usize],
                            (a.0).0[2usize],
                            &mut carry,
                        );
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[0usize],
                            (a.0).0[3usize],
                            &mut carry,
                        );
                        r[4usize + 0usize] = carry;
                        carry = 0;
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[1usize],
                            (a.0).0[2usize],
                            &mut carry,
                        );
                        r[4usize] = fa::mac_with_carry(
                            r[4usize],
                            (a.0).0[1usize],
                            (a.0).0[3usize],
                            &mut carry,
                        );
                        r[4usize + 1usize] = carry;
                        carry = 0;
                        r[5usize] = fa::mac_with_carry(
                            r[5usize],
                            (a.0).0[2usize],
                            (a.0).0[3usize],
                            &mut carry,
                        );
                        r[4usize + 2usize] = carry;
                        carry = 0;
                        r[8usize - 1] = r[8usize - 2] >> 63;
                        r[6usize] = (r[6usize] << 1) | (r[6usize - 1] >> 63);
                        r[5usize] = (r[5usize] << 1) | (r[5usize - 1] >> 63);
                        r[4usize] = (r[4usize] << 1) | (r[4usize - 1] >> 63);
                        r[3usize] = (r[3usize] << 1) | (r[3usize - 1] >> 63);
                        r[2usize] = (r[2usize] << 1) | (r[2usize - 1] >> 63);
                        r[1] <<= 1;
                        r[0usize] = fa::mac_with_carry(
                            r[0usize],
                            (a.0).0[0usize],
                            (a.0).0[0usize],
                            &mut carry,
                        );
                        carry = fa::adc(&mut r[0usize + 1], 0, carry);
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[1usize],
                            (a.0).0[1usize],
                            &mut carry,
                        );
                        carry = fa::adc(&mut r[2usize + 1], 0, carry);
                        r[4usize] = fa::mac_with_carry(
                            r[4usize],
                            (a.0).0[2usize],
                            (a.0).0[2usize],
                            &mut carry,
                        );
                        carry = fa::adc(&mut r[4usize + 1], 0, carry);
                        r[6usize] = fa::mac_with_carry(
                            r[6usize],
                            (a.0).0[3usize],
                            (a.0).0[3usize],
                            &mut carry,
                        );
                        carry = fa::adc(&mut r[6usize + 1], 0, carry);
                        let mut carry2 = 0;
                        let k = r[0usize].wrapping_mul(Self::INV);
                        let mut carry = 0;
                        fa::mac_discard(
                            r[0usize],
                            k,
                            10108024940646105089u64,
                            &mut carry,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[1usize],
                            k,
                            2469829653919213789u64,
                            &mut carry,
                        );
                        r[2usize] =
                            fa::mac_with_carry(r[2usize], k, 0u64, &mut carry);
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            4611686018427387904u64,
                            &mut carry,
                        );
                        carry2 =
                            fa::adc(&mut r[4usize + 0usize], carry, carry2);
                        let k = r[1usize].wrapping_mul(Self::INV);
                        let mut carry = 0;
                        fa::mac_discard(
                            r[1usize],
                            k,
                            10108024940646105089u64,
                            &mut carry,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            k,
                            2469829653919213789u64,
                            &mut carry,
                        );
                        r[3usize] =
                            fa::mac_with_carry(r[3usize], k, 0u64, &mut carry);
                        r[4usize] = fa::mac_with_carry(
                            r[4usize],
                            k,
                            4611686018427387904u64,
                            &mut carry,
                        );
                        carry2 =
                            fa::adc(&mut r[4usize + 1usize], carry, carry2);
                        let k = r[2usize].wrapping_mul(Self::INV);
                        let mut carry = 0;
                        fa::mac_discard(
                            r[2usize],
                            k,
                            10108024940646105089u64,
                            &mut carry,
                        );
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            2469829653919213789u64,
                            &mut carry,
                        );
                        r[4usize] =
                            fa::mac_with_carry(r[4usize], k, 0u64, &mut carry);
                        r[5usize] = fa::mac_with_carry(
                            r[5usize],
                            k,
                            4611686018427387904u64,
                            &mut carry,
                        );
                        carry2 =
                            fa::adc(&mut r[4usize + 2usize], carry, carry2);
                        let k = r[3usize].wrapping_mul(Self::INV);
                        let mut carry = 0;
                        fa::mac_discard(
                            r[3usize],
                            k,
                            10108024940646105089u64,
                            &mut carry,
                        );
                        r[4usize] = fa::mac_with_carry(
                            r[4usize],
                            k,
                            2469829653919213789u64,
                            &mut carry,
                        );
                        r[5usize] =
                            fa::mac_with_carry(r[5usize], k, 0u64, &mut carry);
                        r[6usize] = fa::mac_with_carry(
                            r[6usize],
                            k,
                            4611686018427387904u64,
                            &mut carry,
                        );
                        carry2 =
                            fa::adc(&mut r[4usize + 3usize], carry, carry2);
                        (a.0).0 = r[4usize..].try_into().unwrap();
                    }
                }
            }
            __subtract_modulus(a);
        }

        fn sum_of_products<const M: usize>(a: &[F; M], b: &[F; M]) -> F {
            a.iter().zip(b).map(|(a, b)| *a * b).sum()
        }
    }
    #[inline(always)]
    fn __subtract_modulus(a: &mut F) {
        if a.is_geq_modulus() {
            __sub_with_borrow(
                &mut a.0,
                &BigInt([
                    10108024940646105089u64,
                    2469829653919213789u64,
                    0u64,
                    4611686018427387904u64,
                ]),
            );
        }
    }
    #[inline(always)]
    fn __subtract_modulus_with_carry(a: &mut F, carry: bool) {
        if a.is_geq_modulus() || carry {
            __sub_with_borrow(
                &mut a.0,
                &BigInt([
                    10108024940646105089u64,
                    2469829653919213789u64,
                    0u64,
                    4611686018427387904u64,
                ]),
            );
        }
    }
    #[inline(always)]
    fn __add_with_carry(a: &mut B, b: &B) -> bool {
        use crate::biginteger::arithmetic::adc_for_add_with_carry as adc;
        let mut carry = 0;
        carry = adc(&mut a.0[0usize], b.0[0usize], carry);
        carry = adc(&mut a.0[1usize], b.0[1usize], carry);
        carry = adc(&mut a.0[2usize], b.0[2usize], carry);
        carry = adc(&mut a.0[3usize], b.0[3usize], carry);
        carry != 0
    }
    #[inline(always)]
    fn __sub_with_borrow(a: &mut B, b: &B) -> bool {
        use crate::biginteger::arithmetic::sbb_for_sub_with_borrow as sbb;
        let mut borrow = 0;
        borrow = sbb(&mut a.0[0usize], b.0[0usize], borrow);
        borrow = sbb(&mut a.0[1usize], b.0[1usize], borrow);
        borrow = sbb(&mut a.0[2usize], b.0[2usize], borrow);
        borrow = sbb(&mut a.0[3usize], b.0[3usize], borrow);
        borrow != 0
    }
}
