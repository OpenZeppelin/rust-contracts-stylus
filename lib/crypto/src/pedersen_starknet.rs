//! This module contains Fast Pedersen Hash Function implementation.
//!
//! https://docs.starkware.co/starkex/crypto/pedersen-hash-function.html#constant_points

use crate::{
    arithmetic::uint::{Uint, U256},
    curve::{
        sw::{Affine, Projective, SWCurveConfig},
        AffineRepr, CurveConfig, CurveGroup,
    },
    field::{
        fp::{Fp256, FpParams, LIMBS_256},
        prime::PrimeField,
    },
    fp_from_num, from_num,
};

/// Base Field for [`StarknetCurveConfig`].
pub type Fq = Fp256<FqParam>;

/// Base Field parameters for
/// [`StarknetCurveConfig`].
pub struct FqParam;
impl FpParams<LIMBS_256> for FqParam {
    // The multiplicative generator of Fp.
    const GENERATOR: Fp256<Self> = fp_from_num!("3");
    // Starknet's base field modulus.
    const MODULUS: U256 = from_num!("3618502788666131213697322783095070105623107215331596699973092056135872020481");
}

/// Scalar Field for
/// [`crate::pedersen::instance::starknet::StarknetCurveConfig`].
pub type Fr = Fp256<FrParam>;

/// Scalar Field parameters for
/// [`crate::pedersen::instance::starknet::StarknetCurveConfig`].
pub struct FrParam;
impl FpParams<LIMBS_256> for FrParam {
    // Primitive generator of the multiplicative group of the scalar field.
    // TODO: confirm this
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

    // TODO: confirm this
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

const P0: Affine<StarknetCurveConfig> = Affine::new_unchecked(
    fp_from_num!("2089986280348253421170679821480865132823066470938446095505822317253594081284"),
    fp_from_num!("1713931329540660377023406109199410414810705867260802078187082345529207694986"),
);

const P1: Affine<StarknetCurveConfig> = Affine::new_unchecked(
    fp_from_num!("996781205833008774514500082376783249102396023663454813447423147977397232763"),
    fp_from_num!("1668503676786377725805489344771023921079126552019160156920634619255970485781"),
);

const P2: Affine<StarknetCurveConfig> = Affine::new_unchecked(
    fp_from_num!("2251563274489750535117886426533222435294046428347329203627021249169616184184"),
    fp_from_num!("1798716007562728905295480679789526322175868328062420237419143593021674992973"),
);

const P3: Affine<StarknetCurveConfig> = Affine::new_unchecked(
    fp_from_num!("2138414695194151160943305727036575959195309218611738193261179310511854807447"),
    fp_from_num!("113410276730064486255102093846540133784865286929052426931474106396135072156"),
);

const P4: Affine<StarknetCurveConfig> = Affine::new_unchecked(
    fp_from_num!("2379962749567351885752724891227938183011949129833673362440656643086021394946"),
    fp_from_num!("776496453633298175483985398648758586525933812536653089401905292063708816422"),
);

const SHIFT_POINT: Affine<StarknetCurveConfig> = Affine::new_unchecked(
    fp_from_num!("2089986280348253421170679821480865132823066470938446095505822317253594081284"),
    fp_from_num!("1713931329540660377023406109199410414810705867260802078187082345529207694986"),
);

/// Pedersen hash.
#[derive(Clone, Debug, Default)]
pub struct Pedersen {
    hash: Fq,
    len: u64,
}

impl Pedersen {
    #[must_use]
    #[inline]
    /// Creates a new Pedersen hash instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Hashes the input values and returns the result as `x` coordinate of
    /// the point on the curve.
    ///
    /// # Arguments
    ///
    /// * `mut self` - Pedersen hasher instance.
    /// * `input` - The input values to hash.
    ///
    /// # Panics
    ///
    /// * If [`crate::pedersen::Pedersen::finalize`] panics.
    #[must_use]
    pub fn hash(mut self, input: &[U256]) -> Fq {
        for input in input {
            self.update(*input)
        }
        self.finalize()
    }

    /// Add `input` value to the hash state.
    ///
    /// # Arguments
    ///
    /// * `mut self` - Mutable reference to the Pedersen hasher instance.
    /// * `input` - The input values to update the hasher state.
    pub fn update(&mut self, input: U256) {
        self.hash = pedersen_hash(&self.hash, &Fq::from_bigint(input));
        self.len += 1;
    }

    /// Finalize the hash and return the result.
    ///
    /// # Arguments
    ///
    /// * `self` - Pedersen hasher instance.
    ///
    /// # Panics
    ///
    /// * If one of the input values is higher than
    ///   [`crate::pedersen::params::PedersenParams::FIELD_PRIME`].
    /// * If the input values contains more elements than length of
    ///   [`crate::pedersen::params::PedersenParams::constant_points()`] /
    ///   [`crate::pedersen::params::PedersenParams::N_ELEMENT_BITS_HASH`].
    pub fn finalize(self) -> Fq {
        let fp = pedersen_hash(&self.hash, &self.len.into());
        fp.into()
    }
}

fn pedersen_hash(x: &Fq, y: &Fq) -> Fq {
    let processed_x = process_element(*x, P0.into(), P1.into());
    let processed_y = process_element(*y, P2.into(), P3.into());

    let shift_point: Projective<StarknetCurveConfig> = SHIFT_POINT.into();
    let affine = (processed_x + processed_y + shift_point).into_affine();
    affine.x().unwrap()
}

fn process_element(
    x: Fq,
    p1: Projective<StarknetCurveConfig>,
    p2: Projective<StarknetCurveConfig>,
) -> Projective<StarknetCurveConfig> {
    let x = x.into_bigint();
    let shift = 252 - 4;
    let high_part: Uint<4> = x >> shift;
    // TODO#q: implement subtraction for uint. Don't use ct_wrapping_sub
    let low_part: Uint<4> = x.ct_wrapping_sub(&(high_part << shift));
    let x_high = Fr::from_bigint(high_part);
    let x_low = Fr::from_bigint(low_part);
    p1 * x_low + p2 * x_high
}

#[cfg(test)]
mod test {
    use crate::{
        fp_from_hex,
        pedersen_starknet::{pedersen_hash, Fq},
    };

    #[test]
    fn test_pedersen_hash() {
        let test_data: Vec<(Fq, Fq, Fq)> = vec![
            (
                fp_from_hex!("03d937c035c878245caf64531a5756109c53068da139362728feb561405371cb"),
                fp_from_hex!("0208a0a10250e382e1e4bbe2880906c2791bf6275695e02fbbc6aeff9cd8b31a"),
                fp_from_hex!("030e480bed5fe53fa909cc0f8c4d99b8f9f2c016be4c41e13a4848797979c662"),
            ),
            (
                fp_from_hex!("058f580910a6ca59b28927c08fe6c43e2e303ca384badc365795fc645d479d45"),
                fp_from_hex!("078734f65a067be9bdb39de18434d71e79f7b6466a4b66bbd979ab9e7515fe0b"),
                fp_from_hex!("068cc0b76cddd1dd4ed2301ada9b7c872b23875d5ff837b3a87993e0d9996b87"),
            )
        ];

        for (in1, in2, expected_hash) in test_data {
            assert_eq!(pedersen_hash(&in1, &in2), expected_hash);
        }
    }
}
