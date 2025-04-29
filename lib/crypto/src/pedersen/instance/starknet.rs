//! This module contains the pedersen hash function parameters for Starknet.

use crate::{
    arithmetic::uint::U256,
    curve::{
        sw::{Affine, Projective, SWCurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
    pedersen::params::PedersenParams,
};
#[derive(Clone, Default, PartialEq, Eq)]
/// Starknet's Curve Details.
pub struct StarknetCurveConfig;

/// Base Field for [`StarknetCurveConfig`].
pub type Fq = Fp256<FqParam>;
/// Base Field parameters for [`StarknetCurveConfig`].
pub struct FqParam;

impl FpParams<LIMBS_256> for FqParam {
    // The multiplicative generator of Fp.
    const GENERATOR: Fp256<Self> = fp_from_num!("3");
    // Starknet's base field modulus.
    const MODULUS: U256 = from_num!("3618502788666131213697322783095070105623107215331596699973092056135872020481");
}

/// Scalar Field for [`StarknetCurveConfig`].
pub type Fr = Fp256<FrParam>;
/// Scalar Field parameters for [`StarknetCurveConfig`].
pub struct FrParam;

impl FpParams<LIMBS_256> for FrParam {
    // Primitive generator of the multiplicative group of the scalar field.
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    // The curve's group order (`EC_ORDER`).
    const MODULUS: U256 = from_num!("3618502788666131213697322783095070105526743751716087489154079457884512865583");
}

impl CurveConfig for StarknetCurveConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[0x1];
    const COFACTOR_INV: Fr = Fr::ONE;
}

// https://docs.starkware.co/starkex/crypto/stark-curve.html
const G_GENERATOR_X: Fq = fp_from_num!("874739451078007766457464989774322083649278607533249481151382481072868806602");
const G_GENERATOR_Y: Fq = fp_from_num!("152666792071518830868575557812948353041420400780739481342941381225525861407");

impl SWCurveConfig for StarknetCurveConfig {
    const COEFF_A: Fq = fp_from_num!("1");
    const COEFF_B: Fq = fp_from_num!("3141592653589793238462643383279502884197169399375105820974944592307816406665");
    const GENERATOR: Affine<StarknetCurveConfig> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}
#[derive(Clone, Default, PartialEq, Eq)]
/// Pedersen Hash parameters for Starknet.
pub struct StarknetPedersenParams;

impl PedersenParams<StarknetCurveConfig> for StarknetPedersenParams {
    const FIELD_PRIME: U256 = FqParam::MODULUS;
    const N_ELEMENT_BITS_HASH: usize = 252;
    const P_0: Affine<StarknetCurveConfig> =
		Affine::new_unchecked(
            fp_from_num!("996781205833008774514500082376783249102396023663454813447423147977397232763"),
            fp_from_num!("1668503676786377725805489344771023921079126552019160156920634619255970485781")
        );
    const P_1: Affine<StarknetCurveConfig> =
        Affine::new_unchecked(
            fp_from_num!("2251563274489750535117886426533222435294046428347329203627021249169616184184"),
            fp_from_num!("1798716007562728905295480679789526322175868328062420237419143593021674992973")
        );
    const P_2: Affine<StarknetCurveConfig> =
		Affine::new_unchecked(
            fp_from_num!("2138414695194151160943305727036575959195309218611738193261179310511854807447"),
            fp_from_num!("113410276730064486255102093846540133784865286929052426931474106396135072156")
        );
    const P_3:  Affine<StarknetCurveConfig> =
		Affine::new_unchecked(
            fp_from_num!("2379962749567351885752724891227938183011949129833673362440656643086021394946"),
            fp_from_num!("776496453633298175483985398648758586525933812536653089401905292063708816422")
        );
    const SHIFT_POINT: Affine<StarknetCurveConfig> = Affine::new_unchecked(
			fp_from_num!("2089986280348253421170679821480865132823066470938446095505822317253594081284"),
			fp_from_num!("1713931329540660377023406109199410414810705867260802078187082345529207694986")
		);
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloc::{vec, vec::Vec};

    use proptest::prelude::*;

    use super::*;
    use crate::{
        arithmetic::{uint::from_str_hex, BigInteger},
        fp_from_hex,
        pedersen::{
            instance::starknet::{
                Fq, StarknetCurveConfig, StarknetPedersenParams,
            },
            Pedersen,
        },
    };

    #[test]
    fn correct_bits_hash_length() {
        assert_eq!(StarknetPedersenParams::N_ELEMENT_BITS_HASH, 252);
    }

    #[test]
    fn correct_field_prime() {
        assert_eq!(StarknetPedersenParams::FIELD_PRIME, FqParam::MODULUS);
    }

    #[test]
    fn correct_curve_config() {
        // TODO
        // assert!(U256::from(2).pow(U256::from(251)) < FrParam::MODULUS);
        assert!(FrParam::MODULUS < StarknetPedersenParams::FIELD_PRIME);
    }

    #[test]
    fn correct_shift_point() {
        assert_eq!(StarknetPedersenParams::SHIFT_POINT, Affine::new_unchecked(
            fp_from_num!("2089986280348253421170679821480865132823066470938446095505822317253594081284"),
            fp_from_num!("1713931329540660377023406109199410414810705867260802078187082345529207694986")
        ));
    }

    #[derive(Debug)]
    struct StarknetTestCase {
        input: Vec<U256>,
        expected: Fq,
    }

    #[test]
    fn smoke() {
        // Based on https://github.com/starkware-libs/starkware-crypto-utils/blob/master/test/config/signature_test_data.json.
        let test_cases = vec![
                StarknetTestCase {
                    input: vec![from_str_hex("3d937c035c878245caf64531a5756109c53068da139362728feb561405371cb"), from_str_hex("208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a")],
                    expected: fp_from_hex!("30e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662")
                },
                StarknetTestCase {
                    input: vec![from_str_hex("58f580910a6ca59b28927c08fe6c43e2e303ca384badc365795fc645d479d45"), from_str_hex("78734f65a067be9bdb39de18434d71e79f7b6466a4b66bbd979ab9e7515fe0b")],
                    expected: fp_from_hex!("68cc0b76cddd1dd4ed2301ada9b7c872b23875d5ff837b3a87993e0d9996b87"),
                },
            ];
        for test_case in test_cases {
            let pedersen =
                Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();

            assert_eq!(
                pedersen.hash(&test_case.input),
                test_case.expected,
                "Failed for input: {:?}",
                test_case.input
            );
        }
    }

    #[test]
    #[should_panic = "Pedersen hash failed -- invalid input"]
    fn panics_on_wrong_item() {
        let pedersen =
            Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();
        let input = vec![StarknetPedersenParams::FIELD_PRIME];

        let _ = pedersen.hash(&input);
    }

    #[test]
    #[should_panic = "Pedersen hash failed -- too many elements"]
    fn panics_on_too_many_elements() {
        let pedersen =
            Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();

        let one = U256::from(1u32);
        let input = vec![one, one, one];

        let _ = pedersen.hash(&input);
    }
    /*
    fn proper_values() -> impl Strategy<Value = alloy_primitives::U256> {
        any::<alloy_primitives::U256>().prop_filter(
            "Should be less than `StarknetPedersenParams::FIELD_PRIME`",
            |x| {
                U256::from_bytes_le(&x.to_le_bytes_vec())
                    <= StarknetPedersenParams::FIELD_PRIME
            },
        )
    }

    #[test]
    fn hash() {
        proptest!(|(input in prop::collection::vec(proper_values(), 0..3))| {
                let input = input.iter().map(|x|
        U256::from_bytes_le(&x.to_le_bytes_vec())).collect::<Vec<_>>();

                let pedersen = Pedersen::<StarknetPedersenParams,
        StarknetCurveConfig>::new();         _ = pedersen.hash(&input);
            });
    }*/
}
