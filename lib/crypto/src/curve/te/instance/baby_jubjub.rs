//! [Baby Jubjub Elliptic Curve] parameters.
//!
//! [Baby Jubjub Elliptic Curve]: <https://eips.ethereum.org/EIPS/eip-2494>

use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_num!("995203441582195749578291179787384436505546430278305826713579947235728471134");

const G_GENERATOR_Y: Fq = fp_from_num!("5472060717959818805561601436314318772137091100104008585924551046643952123905");

/// Base Field for [`BabyJubjubConfig`].
pub type Fq = Fp256<BabyJubjubFqParam>;
/// Base Field parameters for [`BabyJubjubConfig`].
pub struct BabyJubjubFqParam;

impl FpParams<LIMBS_256> for BabyJubjubFqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("21888242871839275222246405745257275088548364400416034343698204186575808495617");
}

/// Scalar Field for [`BabyJubjubConfig`].
pub type Fr = Fp256<BabyJubjubFrParam>;
/// Scalar Field parameters for [`BabyJubjubConfig`].
pub struct BabyJubjubFrParam;

impl FpParams<LIMBS_256> for BabyJubjubFrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("2736030358979909402780800718157159386076813972158567259200215660948447373041");
}

/// Baby Jubjub's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BabyJubjubConfig;

impl CurveConfig for BabyJubjubConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("2394026564107420727433200628387514462817212225638746351800188703329891451411");
}

impl TECurveConfig for BabyJubjubConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("168700");
    const COEFF_D: Self::BaseField = fp_from_num!("168696");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for BabyJubjubConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("168698");
    const COEFF_B: Self::BaseField = fp_from_num!("1");
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve::{AffineRepr, CurveGroup, PrimeGroup};

    // ---- Test cases from EIP-2494 (https://eips.ethereum.org/EIPS/eip-2494). ----

    #[test]
    fn test_addition() {
        // Test 1: Addition of two different points
        let x1 = fp_from_num!("17777552123799933955779906779655732241715742912184938656739573121738514868268");
        let y1 = fp_from_num!("2626589144620713026669568689430873010625803728049924121243784502389097019475");
        let p1 = Affine::<BabyJubjubConfig>::new_unchecked(x1, y1);

        let x2 = fp_from_num!("16540640123574156134436876038791482806971768689494387082833631921987005038935");
        let y2 = fp_from_num!("20819045374670962167435360035096875258406992893633759881276124905556507972311");
        let p2 = Affine::<BabyJubjubConfig>::new_unchecked(x2, y2);

        let expected_x = fp_from_num!("7916061937171219682591368294088513039687205273691143098332585753343424131937");
        let expected_y = fp_from_num!("14035240266687799601661095864649209771790948434046947201833777492504781204499");
        let expected =
            Affine::<BabyJubjubConfig>::new_unchecked(expected_x, expected_y);

        let result = (p1 + p2).into_affine();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_doubling() {
        // Test 2: Point doubling
        let x1 = fp_from_num!("17777552123799933955779906779655732241715742912184938656739573121738514868268");
        let y1 = fp_from_num!("2626589144620713026669568689430873010625803728049924121243784502389097019475");
        let p1 = Affine::<BabyJubjubConfig>::new_unchecked(x1, y1);

        let expected_x = fp_from_num!("6890855772600357754907169075114257697580319025794532037257385534741338397365");
        let expected_y = fp_from_num!("4338620300185947561074059802482547481416142213883829469920100239455078257889");
        let expected =
            Affine::<BabyJubjubConfig>::new_unchecked(expected_x, expected_y);

        let result = (p1 + p1).into_affine();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_identity_doubling() {
        // Test 3: Doubling the identity element
        let x1 = fp_from_num!("0");
        let y1 = fp_from_num!("1");
        let identity = Affine::<BabyJubjubConfig>::new_unchecked(x1, y1);

        let result = (identity + identity).into_affine();
        assert_eq!(result, identity);
    }
    #[test]
    fn test_curve_membership() {
        // Test 4: Curve membership
        // Point (0,1) is a point on Baby Jubjub
        let x1 = fp_from_num!("0");
        let y1 = fp_from_num!("1");
        let p1 = Affine::<BabyJubjubConfig>::new_unchecked(x1, y1);
        assert!(p1.is_on_curve());

        // Point (1,0) is not a point on Baby Jubjub
        let x2 = fp_from_num!("1");
        let y2 = fp_from_num!("0");
        let p2 = Affine::<BabyJubjubConfig>::new_unchecked(x2, y2);
        assert!(!p2.is_on_curve());
    }

    #[test]
    #[allow(clippy::similar_names)]
    fn test_base_point_choice() {
        // Test 5: Base point choice
        // Check that the base point B = 8 * G
        let gx = fp_from_num!("995203441582195749578291179787384436505546430278305826713579947235728471134");
        let gy = fp_from_num!("5472060717959818805561601436314318772137091100104008585924551046643952123905");
        let g = Affine::<BabyJubjubConfig>::new_unchecked(gx, gy);

        let expected_bx = fp_from_num!("5299619240641551281634865583518297030282874472190772894086521144482721001553");
        let expected_by = fp_from_num!("16950150798460657717958625567821834550301663161624707787222815936182638968203");
        let expected_b =
            Affine::<BabyJubjubConfig>::new_unchecked(expected_bx, expected_by);

        // Multiply by 8
        let b = g.into_group().mul_bigint(8u64).into_affine();
        assert_eq!(b, expected_b);
    }

    #[test]
    fn test_base_point_order() {
        // Test 6: Base point order
        let bx = fp_from_num!("5299619240641551281634865583518297030282874472190772894086521144482721001553");
        let by = fp_from_num!("16950150798460657717958625567821834550301663161624707787222815936182638968203");
        let b = Affine::<BabyJubjubConfig>::new_unchecked(bx, by);

        // Create scalar l
        let l = fp_from_num!("2736030358979909402780800718157159386076813972158567259200215660948447373041");

        // B * l should equal identity (0, 1)
        let result = (b.into_group() * l).into_affine();

        let identity_x = fp_from_num!("0");
        let identity_y = fp_from_num!("1");
        let identity =
            Affine::<BabyJubjubConfig>::new_unchecked(identity_x, identity_y);

        assert_eq!(result, identity);
    }
}
