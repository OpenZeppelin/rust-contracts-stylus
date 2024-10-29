use crypto_bigint::{Limb, Uint};
use hex_literal::hex;

use crate::field::fp::{Fp256, FpParams};

pub struct FieldParam;
pub type FpVesta = Fp256<FieldParam>;

use crate::{field::fp::Fp, from_hex};

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

const fn from_str_radix<const LIMBS: usize>(
    s: &str,
    radix: u32,
) -> Uint<LIMBS> {
    let bytes = s.as_bytes();
    let mut index = bytes.len() - 1;

    let mut uint = Uint::from_u32(0);
    let mut order = Uint::<LIMBS>::from_u32(1);
    let uint_radix = Uint::<LIMBS>::from_u32(radix);
    loop {
        let ch = parse_utf8_byte(bytes[index]);
        let digit = match ch.to_digit(radix) {
            None => {
                panic!("invalid digit");
            }
            Some(digit) => Uint::<LIMBS>::from_u32(digit),
        };

        let new_uint = mul(&digit, &order);
        uint = add(&uint, &new_uint);

        order = mul(&uint_radix, &order);

        if index == 0 {
            return uint;
        }
        index -= 1;
    }
}

const fn mul<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    b: &Uint<LIMBS>,
) -> Uint<LIMBS> {
    let (low, high) = a.mul_wide(&b);
    assert!(high.bits() == 0, "overflow on multiplication");
    low
}

const fn add<const LIMBS: usize>(
    a: &Uint<LIMBS>,
    b: &Uint<LIMBS>,
) -> Uint<LIMBS> {
    let (low, carry) = a.adc(b, Limb::ZERO);
    assert!(carry.0 == 0, "overflow on addition");
    low
}

const fn parse_utf8_byte(byte: u8) -> char {
    match byte {
        0x00..=0x7F => byte as char,
        _ => panic!("non-ASCII character found"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str_radix() {
        let uint = from_str_radix::<4>("28948022309329048855892746252171976963363056481941647379679742748393362948097", 10);
        let expected = Uint::<4>::from_words([
            10108024940646105089u64,
            2469829653919213789u64,
            0u64,
            4611686018427387904u64,
        ]);
        assert_eq!(uint, expected);
    }
}
