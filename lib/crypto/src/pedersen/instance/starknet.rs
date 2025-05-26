//! This module contains the Pedersen Hash function (and curve) parameters for
//! [Starknet instance].
//!
//! [Starknet instance]: <https://docs.starkware.co/starkex/crypto/pedersen-hash-function.html>

use crate::{
    arithmetic::uint::U256,
    curve::{
        sw::{Affine, SWCurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_hex, from_num,
    pedersen::params::PedersenParams,
};

/// Starknet's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
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
    /// Low part bits.
    const LOW_PART_BITS: u32 = 248;
    /// Low part mask. (2**248 - 1)
    const LOW_PART_MASK: U256 = from_hex!(
        "00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    const N_ELEMENT_BITS_HASH: usize = 252;
    const P_0: Affine<StarknetCurveConfig> = Affine::new_unchecked(
			fp_from_num!("2089986280348253421170679821480865132823066470938446095505822317253594081284"),
			fp_from_num!("1713931329540660377023406109199410414810705867260802078187082345529207694986")
		);
    const P_1: Affine<StarknetCurveConfig> =
		Affine::new_unchecked(
            fp_from_num!("996781205833008774514500082376783249102396023663454813447423147977397232763"),
            fp_from_num!("1668503676786377725805489344771023921079126552019160156920634619255970485781")
        );
    const P_2: Affine<StarknetCurveConfig> =
        Affine::new_unchecked(
            fp_from_num!("2251563274489750535117886426533222435294046428347329203627021249169616184184"),
            fp_from_num!("1798716007562728905295480679789526322175868328062420237419143593021674992973")
        );
    const P_3: Affine<StarknetCurveConfig> =
		Affine::new_unchecked(
            fp_from_num!("2138414695194151160943305727036575959195309218611738193261179310511854807447"),
            fp_from_num!("113410276730064486255102093846540133784865286929052426931474106396135072156")
        );
    const P_4:  Affine<StarknetCurveConfig> =
		Affine::new_unchecked(
            fp_from_num!("2379962749567351885752724891227938183011949129833673362440656643086021394946"),
            fp_from_num!("776496453633298175483985398648758586525933812536653089401905292063708816422")
        );
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use super::*;
    use crate::{
        arithmetic::BigInteger,
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
    fn correct_shift_point() {
        assert_eq!(StarknetPedersenParams::P_0, Affine::new_unchecked(
            fp_from_num!("2089986280348253421170679821480865132823066470938446095505822317253594081284"),
            fp_from_num!("1713931329540660377023406109199410414810705867260802078187082345529207694986")
        ));
    }

    #[derive(Debug)]
    struct StarknetTestCase {
        x: Fq,
        y: Fq,
        expected: Option<Fq>,
    }

    #[test]
    fn smoke() {
        // Based on <https://github.com/starkware-libs/starkware-crypto-utils/blob/master/test/config/signature_test_data.json>.
        let test_cases = vec![
                StarknetTestCase {
                    x: fp_from_hex!("3d937c035c878245caf64531a5756109c53068da139362728feb561405371cb"),
                    y: fp_from_hex!("208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a"),
                    expected: Some(fp_from_hex!("30e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662"))
                },
                StarknetTestCase {
                    x: fp_from_hex!("58f580910a6ca59b28927c08fe6c43e2e303ca384badc365795fc645d479d45"),
                    y: fp_from_hex!("78734f65a067be9bdb39de18434d71e79f7b6466a4b66bbd979ab9e7515fe0b"),
                    expected: Some(fp_from_hex!("68cc0b76cddd1dd4ed2301ada9b7c872b23875d5ff837b3a87993e0d9996b87")),
                },
            ];
        for test_case in test_cases {
            let pedersen =
                Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();

            assert_eq!(
                pedersen.hash(test_case.x, test_case.y),
                test_case.expected,
                "Failed for input x: {:?}, y: {:?}",
                test_case.x,
                test_case.y
            );
        }
    }

    fn from_u256(elem: &alloy_primitives::U256) -> U256 {
        U256::from_bytes_le(&elem.to_le_bytes_vec())
    }

    #[test]
    fn hash() {
        // Check no panics.
        proptest!(|(input1: alloy_primitives::U256, input2: alloy_primitives::U256)| {
            let pedersen =
                Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();
            let hash = pedersen.hash(from_u256(&input1).into(), from_u256(&input2).into());
            assert!(hash.is_some());
        });
    }
}
