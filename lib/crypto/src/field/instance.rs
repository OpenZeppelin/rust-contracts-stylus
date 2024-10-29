use crypto_bigint::{U256, U64};

use crate::{
    field::fp::{Fp256, Fp64, FpParams},
    fp_from_num, from_num,
};

pub type FpVesta = Fp256<VestaParam>;
pub struct VestaParam;
impl FpParams<4> for VestaParam {
    const GENERATOR: Fp256<VestaParam> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("28948022309329048855892746252171976963363056481941647379679742748393362948097");
}

pub type FpBabyBear = Fp64<BabyBearParam>;
pub struct BabyBearParam;
impl FpParams<1> for BabyBearParam {
    const GENERATOR: Fp64<BabyBearParam> = fp_from_num!("31");
    const MODULUS: U64 = from_num!("2013265921");
}

pub type FpBLS12 = Fp256<BLS12Param>;
pub struct BLS12Param;
impl FpParams<4> for BLS12Param {
    const GENERATOR: Fp256<BLS12Param> = fp_from_num!("7");
    const MODULUS: U256 = from_num!("52435875175126190479447740508185965837690552500527637822603658699938581184513");
}

pub type FpBN256 = Fp256<BN256Param>;
pub struct BN256Param;
impl FpParams<4> for BN256Param {
    const GENERATOR: Fp256<BN256Param> = fp_from_num!("7");
    const MODULUS: U256 = from_num!("21888242871839275222246405745257275088548364400416034343698204186575808495617");
}

pub type FpGoldiLocks = Fp64<GoldiLocksParam>;
pub struct GoldiLocksParam;
impl FpParams<1> for GoldiLocksParam {
    const GENERATOR: Fp64<GoldiLocksParam> = fp_from_num!("7");
    const MODULUS: U64 = from_num!("18446744069414584321");
}

pub type FpPallas = Fp256<PallasParam>;
pub struct PallasParam;
impl FpParams<4> for PallasParam {
    const GENERATOR: Fp256<PallasParam> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("28948022309329048855892746252171976963363056481941560715954676764349967630337");
}
