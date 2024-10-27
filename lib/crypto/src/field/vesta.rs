use crypto_bigint::Uint;

use crate::field::fp::{Fp256, FpConfig};

/*
#[derive(MontConfig)]
#[modulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[generator = "5"]
pub struct FqConfig;
*/

pub struct FieldConfig;
pub type FpVesta = Fp256<FieldConfig>;

use crate::field::fp::Fp;

#[automatically_derived]
impl FpConfig<4usize> for FieldConfig {
    const GENERATOR: Fp<FieldConfig, 4> = {
        Fp::new_unchecked(Uint::<4>::from_words([
            12037607305579515999u64,
            11221139188353527881u64,
            11411081306099606126u64,
            3307517586042601304u64,
        ]))
    };
    const MODULUS: Uint<4> = Uint::<4>::from_words([
        10108024940646105089u64,
        2469829653919213789u64,
        0u64,
        4611686018427387904u64,
    ]);
}

/*
#[automatically_derived]
impl FpConfig<4usize> for FieldConfig {
    const GENERATOR: Fp<FieldConfig, 4> = {
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
    const MODULUS: BigInt<4> = BigInt([
        10108024940646105089u64,
        2469829653919213789u64,
        0u64,
        4611686018427387904u64,
    ]);
}
*/
