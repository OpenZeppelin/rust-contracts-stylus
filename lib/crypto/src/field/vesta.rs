use crypto_bigint::Uint;

use crate::field::fp::{Fp256, FpParams};

pub struct FieldParam;
pub type FpVesta = Fp256<FieldParam>;

use crate::field::fp::Fp;

// TODO#q: Use proc macro or function macro
//  Can look smth like this:
/*
#[derive(MontConfig)]
#[modulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[generator = "5"]
pub struct FqConfig;
*/
#[automatically_derived]
impl FpParams<4usize> for FieldParam {
    const GENERATOR: Fp<FieldParam, 4> = {
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
