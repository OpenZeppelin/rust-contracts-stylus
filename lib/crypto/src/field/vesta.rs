use crate::field::fp::{Fp256, MontBackend, MontConfig};

/*
#[derive(MontConfig)]
#[modulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[generator = "5"]
pub struct FqConfig;
*/

pub struct FqConfig;
pub type FpVesta = Fp256<MontBackend<FqConfig, 4>>;

use crate::{field::fp::Fp, BigInt};
#[automatically_derived]
impl MontConfig<4usize> for FqConfig {
    const GENERATOR: Fp<MontBackend<FqConfig, 4>, 4> = {
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
