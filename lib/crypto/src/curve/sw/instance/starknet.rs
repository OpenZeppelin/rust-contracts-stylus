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

#[cfg(test)]
mod test {
    use num_traits::Zero;

    use crate::{
        curve::{
            sw::{instance::starknet::StarknetCurveConfig, Affine, Projective},
            AffineRepr, CurveGroup,
        },
        field::group::AdditiveGroup,
        fp_from_hex,
    };

    // Values generated with "algebra" implementation of starknet curve.
    // https://github.com/arkworks-rs/algebra/blob/48ec86ef03f700244a5a24d38a751959ab64fd3e/curves/ed_on_bls12_381_bandersnatch/src/curves/mod.rs#L53

    #[test]
    fn scalar_mul() {
        assert!(
            Affine::<StarknetCurveConfig>::generator()
                .mul_bigint(0u32)
                .into_affine()
                .infinity
        );

        let result: Vec<_> = (1u32..25)
            .map(|k| {
                Affine::<StarknetCurveConfig>::generator()
                    .mul_bigint(k)
                    .into_affine()
            })
            .collect();

        let expected =
            [
                (fp_from_hex!("1EF15C18599971B7BECED415A40F0C7DEACFD9B0D1819E03D723D8BC943CFCA"), fp_from_hex!("5668060AA49730B7BE4801DF46EC62DE53ECD11ABE43A32873000C36E8DC1F")),
                (fp_from_hex!("759CA09377679ECD535A81E83039658BF40959283187C654C5416F439403CF5"), fp_from_hex!("6F524A3400E7708D5C01A28598AD272E7455AA88778B19F93B562D7A9646C41")),
                (fp_from_hex!("411494B501A98ABD8262B0DA1351E17899A0C4EF23DD2F96FEC5BA847310B20"), fp_from_hex!("7E1B3EBAC08924D2C26F409549191FCF94F3BF6F301ED3553E22DFB802F0686")),
                (fp_from_hex!("A7DA05A4D664859CCD6E567B935CDFBFE3018C7771CB980892EF38878AE9BC"), fp_from_hex!("584B0C2BC833A4C88D62B387E0EF868CAE2EAAA288F4CA7B34C84B46CA031B6")),
                (fp_from_hex!("788435D61046D3EEC54D77D25BD194525F4FA26EBE6575536BC6F656656B74C"), fp_from_hex!("13926386B9E5E908C359519EAA68C44A2430F4B4CA5D0DBDCB4231F031EB18B")),
                (fp_from_hex!("1EFC3D7C9649900FCBD03F578A8248D095BC4B6A13B3C25F9886EF971FF96FA"), fp_from_hex!("694E4DCE951394737CF62C7AB0946D5A64940F7B9E573F4324C1D6CE9C4D991")),
                (fp_from_hex!("743829E0A179F8AFE223FC8112DFC8D024AB6B235FD42283C4F5970259CE7B7"), fp_from_hex!("E67A0A63CC493225E45B9178A3375596EA2A1D7012628A328DBC14C78CD1B7")),
                (fp_from_hex!("6EEEE2B0C71D681692559735E08A2C3BA04E7347C0C18D4D49B83BB89771591"), fp_from_hex!("72498C69F16E02231E26A6A6ACAABB8714E0AF1306066231DD38C233EE15216")),
                (fp_from_hex!("216B4F076FF47E03A05032D1C6EE17933D8DE8B2B4C43EB5AD5A7E1B25D3849"), fp_from_hex!("54B14E088019C05FD3C7EA1DADEF2999DE50590264FBF9FFE77692CEB241A8C")),
                (fp_from_hex!("320CEAE3120E56F6006F7D626760F12FC276A3C7683E9B0B87C097D7BE8DBDE"), fp_from_hex!("760D1688317BE9E2CF74EAA8B2D39E886A6C18A223A9CCC9B19429F2A174377")),
                (fp_from_hex!("408F052DFE0289AB18A69FFDCB38A303FB0766979DCA58262A9FABE4A0C7632"), fp_from_hex!("6342E6ABA72FD3FEF06D8274BDAF8CC86BC3EAB59B3AE413F9E3CB086317923")),
                (fp_from_hex!("66276B22EDF076517B8FA9287280242555AFDA9ED00E78EEDC9F99BE8542AA3"), fp_from_hex!("587D4C16426BC2B24C41319CA321CC7325359AD3C9BC465E30EB9078A5C002A")),
                (fp_from_hex!("55B1D8AB7FA62903691A92EECBEAD7205F512FC27CE1EC2DB7120643585B23D"), fp_from_hex!("4A5EDFF810FB09724E54ABBCF3AD2B7A361C14440A43294021A03138701E89F")),
                (fp_from_hex!("3ED4A45432B30FB5F765BE330E5D5766D54E78C24F50A804F998CB6B043BB4C"), fp_from_hex!("5C05669920DD017729DF6116C0D616B5884DFF28F158E365465016466630DC9")),
                (fp_from_hex!("64B098AB256881BB3916F719B8C1E362B9C681446D454773D513E647F0B148D"), fp_from_hex!("67861383870079FA6F3F3AF8083D0A0D84D6F38368E0A220B521D47359FF8CE")),
                (fp_from_hex!("B582A82E6C8AD99E38FCBD2A4DA97B37D0CDB7D776EDB84A661D79EC4824AC"), fp_from_hex!("7CF4349849204906E1B19538552FCB1565171AB453694B574C87E2A35206D9C")),
                (fp_from_hex!("78406570D44F1293762FD99F7E42B034A8A5973542A990A1D1F35C52EDF85EF"), fp_from_hex!("3172C9AF27AD47F777B89DF0E215B7655A436A3D3D7A1C0A573F29F3146C1F0")),
                (fp_from_hex!("19661066E96A8B9F06A1D136881EE924DFB6A885239CAA5FD3F87A54C6B25C4"), fp_from_hex!("5C857697280A06CBD3E29ECE3DC730F5CE700E9276A83B8BD38B1E199044689")),
                (fp_from_hex!("4BFAD94C8EAA1D5281D9699D0217A69DE2F432164F5837B2313C807D3123123"), fp_from_hex!("2B8A6F5F1A15D9AEBFA70820EFB26114F05D3ACAC2057FB3E44F8B497CB8D12")),
                (fp_from_hex!("A32275BBDB8AF5280E7290B75CEFD515748A41F37C404CFC34854607EE8815"), fp_from_hex!("1F6A68BD0480B8A5BE7BBC6B77C3F8371A5AC184B8174AE32A12B0F0BE3DB85")),
                (fp_from_hex!("53012CF1B98B09564CC6C6264ED60932EF373AEB9833BC9F2436EE052F40273"), fp_from_hex!("CE9EF25B1770EFD7A633CE12B9D8F948AC523C92E03E080EA32FA85473840")),
                (fp_from_hex!("6658AB3F9FA93331CAB39626214CEC2D4BDC98B6C7876E80D10EEA285F05AAC"), fp_from_hex!("3D11FED8EAA0D453A173D8970F29458D8D4FDFF1A6CEB440B33D3F9FAFB4255")),
                (fp_from_hex!("2A862A05B8B7F23D870E4A52234A11AB9A5558C2E7BA991ACB6549EC23E4E69"), fp_from_hex!("A4AFA230894CCD0FD93AFA25E950EF6E613413A536E002A3DD2456811868E0")),
                (fp_from_hex!("D94DC52F39A3342889B7D871CA2CAFBCA3D1D673B94F1DD167AE28E626F23B"), fp_from_hex!("200D25AC8EF06553423DD4353328FBCAC4C2772C45904C063E9F1E897103E10")),
            ];

        for (result, (expected_x, expected_y)) in result.iter().zip(expected) {
            assert!(result.is_on_curve());
            assert_eq!(result.x, expected_x);
            assert_eq!(result.y, expected_y);
        }
    }

    #[test]
    fn point_add() {
        let g = Affine::<StarknetCurveConfig>::generator();
        let g_proj: Projective<StarknetCurveConfig> = g.into();

        // Test G + G = 2G
        let expected_g2 = Affine::new_unchecked(
            fp_from_hex!("759CA09377679ECD535A81E83039658BF40959283187C654C5416F439403CF5"),
            fp_from_hex!("6F524A3400E7708D5C01A28598AD272E7455AA88778B19F93B562D7A9646C41"),
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
        let g = Affine::<StarknetCurveConfig>::generator();
        let g_proj: Projective<StarknetCurveConfig> = g.into();

        // Test G - G = 0
        let zero = g_proj - g_proj;
        assert!(zero.is_zero());

        // Test 2G - G = G
        let g2: Projective<StarknetCurveConfig> = Affine::new_unchecked(
            fp_from_hex!("759CA09377679ECD535A81E83039658BF40959283187C654C5416F439403CF5"),
            fp_from_hex!("6F524A3400E7708D5C01A28598AD272E7455AA88778B19F93B562D7A9646C41"),
        ).into();
        assert_eq!(g2 - g_proj, g_proj);
    }
}
