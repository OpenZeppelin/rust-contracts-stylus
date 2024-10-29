use crypto_bigint::U256;

use crate::{
    field::fp::{Fp256, FpParams},
    fp_from_num, from_num,
};

pub type FpVesta = Fp256<FieldParam>;

pub struct FieldParam;
impl FpParams<4> for FieldParam {
    const GENERATOR: Fp256<FieldParam> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("28948022309329048855892746252171976963363056481941647379679742748393362948097");
}
