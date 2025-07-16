//! This module contains the [Starknet curve] configuration.
//!
//! [Starknet curve]: <https://docs.starkware.co/starkex/crypto/stark-curve.html>
use crate::{
    arithmetic::uint::U256,
    curve::{
        sw::{Affine, SWCurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_num!("874739451078007766457464989774322083649278607533249481151382481072868806602");
const G_GENERATOR_Y: Fq = fp_from_num!("152666792071518830868575557812948353041420400780739481342941381225525861407");

/// Base Field for [`StarknetCurveConfig`].
pub type Fq = Fp256<StarknetFqParam>;
/// Base Field parameters for [`StarknetCurveConfig`].
pub struct StarknetFqParam;

impl FpParams<LIMBS_256> for StarknetFqParam {
    // The multiplicative generator of Fp.
    const GENERATOR: Fp256<Self> = fp_from_num!("3");
    // Starknet's base field modulus.
    const MODULUS: U256 = from_num!("3618502788666131213697322783095070105623107215331596699973092056135872020481");
}

/// Scalar Field for [`StarknetCurveConfig`].
pub type Fr = Fp256<StarknetFrParam>;
/// Scalar Field parameters for [`StarknetCurveConfig`].
pub struct StarknetFrParam;

impl FpParams<LIMBS_256> for StarknetFrParam {
    // Primitive generator of the multiplicative group of the scalar field.
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    // The curve's group order (`EC_ORDER`).
    const MODULUS: U256 = from_num!("3618502788666131213697322783095070105526743751716087489154079457884512865583");
}

/// Starknet's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct StarknetCurveConfig;

impl CurveConfig for StarknetCurveConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[0x1];
    const COFACTOR_INV: Fr = Fr::ONE;
}

impl SWCurveConfig for StarknetCurveConfig {
    const COEFF_A: Fq = fp_from_num!("1");
    const COEFF_B: Fq = fp_from_num!("3141592653589793238462643383279502884197169399375105820974944592307816406665");
    const GENERATOR: Affine<StarknetCurveConfig> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}
