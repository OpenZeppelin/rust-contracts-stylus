#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U256;
use ark_ff::{
    AdditiveGroup, BigInteger, Field, Fp256, MontBackend, MontConfig,
    PrimeField,
};
use stylus_sdk::prelude::{entrypoint, public, storage};

pub struct FqConfig;

const _: () = {
    use ark_ff::{
        biginteger::arithmetic as fa,
        fields::{Fp, *},
        BigInt, BigInteger,
    };
    type B = BigInt<4usize>;
    type F = Fp<MontBackend<FqConfig, 4usize>, 4usize>;
    #[automatically_derived]
    impl MontConfig<4usize> for FqConfig {
        const GENERATOR: F = ark_ff::MontFp!("7");
        const MODULUS: B = BigInt([
            4891460686036598785u64,
            2896914383306846353u64,
            13281191951274694749u64,
            3486998266802970665u64,
        ]);
        const TWO_ADIC_ROOT_OF_UNITY: F = ark_ff::MontFp!("1748695177688661943023146337482803886740723238769601073607632802312037301404" );

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
                            4891460686036598785u64,
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
                            2896914383306846353u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[0usize],
                            &mut carry1,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[2usize],
                            k,
                            13281191951274694749u64,
                            &mut carry2,
                        );
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[0usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            3486998266802970665u64,
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
                            4891460686036598785u64,
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
                            2896914383306846353u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[1usize],
                            &mut carry1,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[2usize],
                            k,
                            13281191951274694749u64,
                            &mut carry2,
                        );
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[1usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            3486998266802970665u64,
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
                            4891460686036598785u64,
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
                            2896914383306846353u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[2usize],
                            &mut carry1,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[2usize],
                            k,
                            13281191951274694749u64,
                            &mut carry2,
                        );
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[2usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            3486998266802970665u64,
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
                            4891460686036598785u64,
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
                            2896914383306846353u64,
                            &mut carry2,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[2usize],
                            (a.0).0[2usize],
                            (b.0).0[3usize],
                            &mut carry1,
                        );
                        r[1usize] = fa::mac_with_carry(
                            r[2usize],
                            k,
                            13281191951274694749u64,
                            &mut carry2,
                        );
                        r[3usize] = fa::mac_with_carry(
                            r[3usize],
                            (a.0).0[3usize],
                            (b.0).0[3usize],
                            &mut carry1,
                        );
                        r[2usize] = fa::mac_with_carry(
                            r[3usize],
                            k,
                            3486998266802970665u64,
                            &mut carry2,
                        );
                        r[4usize - 1] = carry1 + carry2;
                        (a.0).0 = r;
                    }
                }
            }
            __subtract_modulus(a);
        }
    }

    #[inline(always)]
    fn __subtract_modulus(a: &mut F) {
        if a.is_geq_modulus() {
            __sub_with_borrow(
                &mut a.0,
                &BigInt([
                    4891460686036598785u64,
                    2896914383306846353u64,
                    13281191951274694749u64,
                    3486998266802970665u64,
                ]),
            );
        }
    }

    #[inline(always)]
    fn __sub_with_borrow(a: &mut B, b: &B) -> bool {
        use ark_ff::biginteger::arithmetic::sbb_for_sub_with_borrow as sbb;
        let mut borrow = 0;
        borrow = sbb(&mut a.0[0usize], b.0[0usize], borrow);
        borrow = sbb(&mut a.0[1usize], b.0[1usize], borrow);
        borrow = sbb(&mut a.0[2usize], b.0[2usize], borrow);
        borrow = sbb(&mut a.0[3usize], b.0[3usize], borrow);
        borrow != 0
    }
};

pub type FpBN256 = Fp256<MontBackend<FqConfig, 4>>;

#[entrypoint]
#[storage]
struct PoseidonExample {}

#[public]
impl PoseidonExample {
    pub fn hash(&mut self, inputs: [U256; 2]) -> Result<U256, Vec<u8>> {
        let inputs: Vec<_> = inputs
            .iter()
            .map(|input| {
                FpBN256::from_le_bytes_mod_order(&input.to_le_bytes_vec())
            })
            .collect();

        let mut res = FpBN256::ONE;
        for _ in 0..1000 {
            for input in inputs.iter() {
                res *= input;
                res.square_in_place();
            }
        }

        let res = res.into_bigint().to_bytes_le();

        Ok(U256::from_le_slice(&res))
    }
}
