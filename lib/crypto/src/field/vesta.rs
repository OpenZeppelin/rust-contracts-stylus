use core::convert::TryInto;

use ark_ff::fields::{Fp256, MontBackend, MontConfig};

#[derive(MontConfig)]
#[modulus = "28948022309329048855892746252171976963363056481941647379679742748393362948097"]
#[generator = "5"]
pub struct FqConfig;
pub type FpVesta = Fp256<MontBackend<FqConfig, 4>>;
