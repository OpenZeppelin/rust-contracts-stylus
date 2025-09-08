//! This module contains the [Curve25519] configuration.
//!
//! [Curve25519]: <https://www.rfc-editor.org/rfc/rfc7748>
use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_num, from_num,
};

const G_GENERATOR_X: Fq =
    fp_from_num!("15112221349535400772501151409588531511454012693041857206046113283949847762202");

const G_GENERATOR_Y: Fq =
    fp_from_num!("46316835694926478169428394003475163141307993866256225615783033603165251855960");

/// Base Field for [`Curve25519Config`].
pub type Fq = Fp256<Curve25519FqParam>;
/// Base Field parameters for [`Curve25519Config`].
pub struct Curve25519FqParam;

impl FpParams<LIMBS_256> for Curve25519FqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("57896044618658097711785492504343953926634992332820282019728792003956564819949");
}

/// Scalar Field for [`Curve25519Config`].
pub type Fr = Fp256<Curve25519FrParam>;
/// Scalar Field parameters for [`Curve25519Config`].
pub struct Curve25519FrParam;

impl FpParams<LIMBS_256> for Curve25519FrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("2");
    const MODULUS: U256 = from_num!("7237005577332262213973186563042994240857116359379907606001950938285454250989");
}

/// Curve25519's Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Curve25519Config;

impl CurveConfig for Curve25519Config {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[8];
    const COFACTOR_INV: Fr = fp_from_num!("2713877091499598330239944961141122840321418634767465352250731601857045344121");
}

impl TECurveConfig for Curve25519Config {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("1").ct_neg();
    const COEFF_D: Self::BaseField = fp_from_num!("37095705934669439343138083508754565189542113879843219016388785533085940283555");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for Curve25519Config {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("486662");
    const COEFF_B: Self::BaseField = fp_from_num!("57896044618658097711785492504343953926634992332820282019728792003956564333285");
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use num_traits::Zero;
    use proptest::{arbitrary::any, prelude::prop, proptest};

    use crate::{
        curve::{
            te::{instance::curve25519::Curve25519Config, Affine, Projective},
            AffineRepr, CurveGroup, PrimeGroup,
        },
        field::group::AdditiveGroup,
        fp_from_hex, fp_from_num,
    };

    // Values generated with "algebra" implementation of curve25519.
    // https://github.com/arkworks-rs/algebra/blob/48ec86ef03f700244a5a24d38a751959ab64fd3e/curves/curve25519/src/curves/mod.rs#L16

    #[test]
    fn scalar_mul() {
        assert!(Affine::<Curve25519Config>::generator()
            .mul_bigint(0u32)
            .into_affine()
            .is_zero());

        let result: Vec<_> = (1u32..25)
            .map(|k| {
                Affine::<Curve25519Config>::generator()
                    .mul_bigint(k)
                    .into_affine()
            })
            .collect();

        let expected = [
            (fp_from_hex!("216936D3CD6E53FEC0A4E231FDD6DC5C692CC7609525A7B2C9562D608F25D51A"), fp_from_hex!("6666666666666666666666666666666666666666666666666666666666666658")),
            (fp_from_hex!("36AB384C9F5A046C3D043B7D1833E7AC080D8E4515D7A45F83C5A14E2843CE0E"), fp_from_hex!("2260CDF3092329C21DA25EE8C9A21F5697390F51643851560E5F46AE6AF8A3C9")),
            (fp_from_hex!("67AE9C4A22928F491FF4AE743EDAC83A6343981981624886AC62485FD3F8E25C"), fp_from_hex!("1267B1D177EE69ABA126A18E60269EF79F16EC176724030402C3684878F5B4D4")),
            (fp_from_hex!("203DA8DB56CFF1468325D4B87A3520F91A739EC193CE1547493AA657C4C9F870"), fp_from_hex!("47D0E827CB1595E1470EB88580D5716C4CF22832EA2F0FF0DF38AB61CA32112F")),
            (fp_from_hex!("49FDA73EADE3587BFCEF7CF7D12DA5DE5C2819F93E1BE1A591409CC0322EF233"), fp_from_hex!("5F4825B298FEAE6FE02C6E148992466631282ECA89430B5D10D21F83D676C8ED")),
            (fp_from_hex!("4C9797BA7A45601C62AEACC0DD0A29BEA1E599826C7B4427783A741A7DCBF23D"), fp_from_hex!("54DE3FC2886D8A11DB709A7FD4F7D77F9417C06944D6B60C1D27AD0F9497EF4")),
            (fp_from_hex!("14568685FCF4BD4EE9E3EE194B1D810783E809F3BBF1CE955855981AF50E4107"), fp_from_hex!("31C563E32B47D52F87CE6468DD36AD41F0882B46F7ABF23D12C4C4B59F4062B8")),
            (fp_from_hex!("6742E15F97D771B642862D5CF84ECF93EB3AC67B80698B993B87FDBC08A584C8"), fp_from_hex!("21D30600C9E573796EAD6F09668AF38F81783CFC621EE4931E2F5BA9FC37B9B4")),
            (fp_from_hex!("357CC970C80071651BF336E06F9422B886D80E5C2E4E0294D3E023065185715C"), fp_from_hex!("7F3D23C2C2DD0DF4B2BEFCE956F2D2FD1F789013236E4430C74E44845522F1C0")),
            (fp_from_hex!("602C797E30CA6D754470B60ED2BC8677207E8E4ED836F81444951F224877F94F"), fp_from_hex!("637FFCAA7A1B2477C8E44D54C898BFCF2576A6853DE0E843BA8874B06AE87B2C")),
            (fp_from_hex!("14E528B1154BE417B6CF078DD6712438D381A5B2C593D552FF2FD2C1207CF3CB"), fp_from_hex!("2D9082313F21AB975A6F7CE340FF0FCE1258591C3C9C58D4308F2DC36A033713")),
            (fp_from_hex!("4719E17E016E5D355ECF70E00CA249DB3295BF2385C13B42AE62FE6678F0902D"), fp_from_hex!("4070CE608BCE8022E71D6C4E637825B856487EB45273966733D281DC2E2DE4F9")),
            (fp_from_hex!("107427E0D5F366CCDB33ADF0282D304F8843E3E88D22B7B83780E073B7C05FED"), fp_from_hex!("12DBB00DED538B7478466022D2DA89B83740CFB2289A272387EFE1AEEA401F80")),
            (fp_from_hex!("205F3B42F5884AAF048C7A895CCABB15D8DEE6D83E39832AA38E7353B58515B9"), fp_from_hex!("4E50256F50C4CB8115BAD17ACBB702BFA74898E819B6265C8369FD98899C2839")),
            (fp_from_hex!("4F162DEAEC2EC435DC5AC6F95D20419ED9631374770189CB90617F3E66A18DC1"), fp_from_hex!("12CBFB2D04FF22F55162F70164D29331ACE5AF18A19A9AA1946D4CC4AD2E5CDF")),
            (fp_from_hex!("23A4860627E53AEEB8E22B1508249C9109578D33E7BF237459B2596D6C28F9F8"), fp_from_hex!("709696F2827FC3729F980F2E3AAD6E78B06A11FF8E079C27D87AAB37C16727EB")),
            (fp_from_hex!("7DC52D5A7DB816E9B850741EA2FD72918D94985B85A20B4DC5597853A876DF6A"), fp_from_hex!("6F6D2BCA60003EF9F24AC245CC919FB717B188723B34F901CD6CFE9BEC97BE04")),
            (fp_from_hex!("1368877F4867292AAF9C0393BC2B0E869158987876B8001297B644A64BB10B96"), fp_from_hex!("2E1126847E0BD8987DE8E8EA8A96C3A5BC810E4ED6D496B0354E3E90E075B04A")),
            (fp_from_hex!("1D81F74A5BA45C7022E8C140D763B9C1B0E281A5304696E74F791A3A04A94472"), fp_from_hex!("3F185A93D95A4347227C5BB6DDD65CF42E1830823F435F3083FE6102691D55B9")),
            (fp_from_hex!("673C65CAEDD698B94F5BBD757DF73A9E6985150ECD4A2135A058E273AB4CF9AF"), fp_from_hex!("136CEBACB6260A9D5E6A3E3171C535F0BE71CFBE16A960B9DD317BDA6F3C5A38")),
            (fp_from_hex!("6F0AC78E5EB90E87958588F9D47541EDF252CB1DDE3D073CC45E3E7EF9365716"), fp_from_hex!("6628D116B7975AE5F323E5DDF4F8CC35AE06D5C5C7D8A56EFFC66051336D289E")),
            (fp_from_hex!("1E029B938C915F04B0C73D7338516AD51E376A9AFA7DE7C8C077622C2AEC2F7A"), fp_from_hex!("6BFC9472CDE96427C4AC03F52E0D2B3CDCE6566535DCEE5A85A6A44B8975F24")),
            (fp_from_hex!("2188AC423C67DB5625915E05222A391BCAF91F05D9B7CC2CAB5798B2D2E14D95"), fp_from_hex!("23240C559C57B79A4DF69A23FC46E50504277B1FA49369AB663D79782B33C0EE")),
            (fp_from_hex!("70985F28875D4006E0968D9C952D799E610ED8E052A9A10E9677C71EE8886B81"), fp_from_hex!("604E1B93C877B9896DCA33CF8A2093CDDF9FD21208C20D08E7B2444FED7B79F1")),
        ];

        for (result, (expected_x, expected_y)) in result.iter().zip(expected) {
            assert!(result.is_on_curve());
            assert_eq!(result.x, expected_x);
            assert_eq!(result.y, expected_y);
        }
    }

    #[test]
    fn point_add() {
        let g = Affine::<Curve25519Config>::generator();
        let g_proj: Projective<Curve25519Config> = g.into();

        // Test G + G = 2G
        let expected_g2 = Affine::new_unchecked(
            fp_from_hex!("36AB384C9F5A046C3D043B7D1833E7AC080D8E4515D7A45F83C5A14E2843CE0E"),
            fp_from_hex!("2260CDF3092329C21DA25EE8C9A21F5697390F51643851560E5F46AE6AF8A3C9"),
        );
        let g2 = g_proj + g;
        let g2_affine = g2.into_affine();
        assert_eq!(g2_affine, expected_g2);
        let g2_affine = g_proj.double().into_affine();
        assert_eq!(g2_affine, expected_g2);

        // Test G + (-G) = 0
        let neg_g = -g_proj;
        let zero = g_proj + neg_g;
        assert!(zero.is_zero());
    }

    #[test]
    fn point_sub() {
        let g = Affine::<Curve25519Config>::generator();
        let g_proj: Projective<Curve25519Config> = g.into();

        // Test G - G = 0
        let zero = g_proj - g_proj;
        assert!(zero.is_zero());

        // Test 2G - G = G
        let g2: Projective<Curve25519Config> = Affine::new_unchecked(
            fp_from_hex!("36AB384C9F5A046C3D043B7D1833E7AC080D8E4515D7A45F83C5A14E2843CE0E"),
            fp_from_hex!("2260CDF3092329C21DA25EE8C9A21F5697390F51643851560E5F46AE6AF8A3C9"),
        ).into();
        assert_eq!(g2 - g_proj, g_proj);
    }

    #[test]
    fn normalize_batch() {
        // Larger collection makes test pass noticeably longer.
        proptest!(|(scalars in prop::collection::vec(any::<u32>(), 1..10))|{
            let prj_points: Vec<_> = scalars.iter()
                .map(|&k| Affine::<Curve25519Config>::generator().mul_bigint(k))
                .collect();

            let expected_aff_points: Vec<_> =
                prj_points.iter().map(|prj| prj.into_affine()).collect();

            let aff_points =
                Projective::<Curve25519Config>::normalize_batch(&prj_points);

            assert_eq!(aff_points, expected_aff_points);
        });
    }

    #[test]
    #[should_panic = "projective Z coordinate should not be zero"]
    fn normalize_batch_should_panic_on_invalid_point() {
        let prj_points = vec![
            Projective::<Curve25519Config>::generator(),
            Projective::<Curve25519Config>::generator().mul_bigint(2u32),
            Projective::<Curve25519Config>::generator().mul_bigint(3u32),
            Projective::<Curve25519Config>::new_unchecked(
                fp_from_num!("1"),
                fp_from_num!("1"),
                fp_from_num!("1"),
                fp_from_num!("0"),
            ),
        ];

        let _ = Projective::<Curve25519Config>::normalize_batch(&prj_points);
    }
}
