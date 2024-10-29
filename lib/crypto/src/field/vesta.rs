use crypto_bigint::{Limb, Uint};
use hex_literal::hex;

use crate::field::fp::{Fp256, FpParams};

pub struct FieldParam;
pub type FpVesta = Fp256<FieldParam>;

use crate::{bigint::from_str_radix, field::fp::Fp, from_hex};

// TODO#q: Use proc macro or function macro
//  Can look smth like this:
/*
#[derive(MontConfig)]
#[modulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[generator = "5"]
pub struct FqConfig;
*/
impl FpParams<4> for FieldParam {
    const GENERATOR: Fp<FieldParam, 4> = Fp::new(from_str_radix("5", 10));
    const MODULUS: Uint<4> = from_str_radix("28948022309329048855892746252171976963363056481941647379679742748393362948097", 10);
}
