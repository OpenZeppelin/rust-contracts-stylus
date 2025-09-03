//! [Bandersnatch Elliptic Curve] parameters.
//!
//! [Bandersnatch Elliptic Curve]: <https://eprint.iacr.org/2021/1152>

use crate::{
    arithmetic::uint::U256,
    curve::{
        te::{Affine, MontCurveConfig, TECurveConfig},
        CurveConfig,
    },
    field::fp::{Fp256, FpParams, LIMBS_256},
    fp_from_hex, fp_from_num, from_num,
};

const G_GENERATOR_X: Fq = fp_from_hex!(
    "29c132cc2c0b34c5743711777bbe42f32b79c022ad998465e1e71866a252ae18"
);
const G_GENERATOR_Y: Fq = fp_from_hex!(
    "2a6c669eda123e0f157d8b50badcd586358cad81eee464605e3167b6cc974166"
);

/// Base Field for [`BandersnatchConfig`].
pub type Fq = Fp256<BandersnatchFqParam>;
/// Base Field parameters for [`BandersnatchConfig`].
pub struct BandersnatchFqParam;

impl FpParams<LIMBS_256> for BandersnatchFqParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("52435875175126190479447740508185965837690552500527637822603658699938581184513");
}

/// Scalar Field for [`BandersnatchConfig`].
pub type Fr = Fp256<BandersnatchFrParam>;
/// Scalar Field parameters for [`BandersnatchConfig`].
pub struct BandersnatchFrParam;

impl FpParams<LIMBS_256> for BandersnatchFrParam {
    const GENERATOR: Fp256<Self> = fp_from_num!("5");
    const MODULUS: U256 = from_num!("13108968793781547619861935127046491459309155893440570251786403306729687672801");
}

/// Bandersnatch Curve Details.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct BandersnatchConfig;

impl CurveConfig for BandersnatchConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    const COFACTOR: &'static [u64] = &[4];
    const COFACTOR_INV: Fr = fp_from_num!("9831726595336160714896451345284868594481866920080427688839802480047265754601");
}

impl TECurveConfig for BandersnatchConfig {
    type MontCurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_num!("5").ct_neg();
    const COEFF_D: Self::BaseField =
        fp_from_num!("45022363124591815672509500913686876175488063829319466900776701791074614335719");
    const GENERATOR: Affine<Self> =
        Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
}

impl MontCurveConfig for BandersnatchConfig {
    type TECurveConfig = Self;

    const COEFF_A: Self::BaseField = fp_from_hex!(
        "4247698f4e32ad45a293959b4ca17afa4a2d2317e4c6ce5023e1fd63d1b5de98"
    );
    const COEFF_B: Self::BaseField = fp_from_hex!(
        "300c3385d13bedb7c9e229e185c4ce8b1dd3b71366bb97c30855c0aa41d62727"
    );
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use num_traits::Zero;

    use crate::{
        curve::{
            te::{
                instance::bandersnatch::BandersnatchConfig, Affine, Projective,
            },
            AffineRepr, CurveGroup,
        },
        field::group::AdditiveGroup,
        fp_from_hex,
    };

    // Values generated with "algebra" implementation of Bandersnatch curve.
    // https://github.com/arkworks-rs/algebra/blob/48ec86ef03f700244a5a24d38a751959ab64fd3e/curves/ed_on_bls12_381_bandersnatch/src/curves/mod.rs#L53

    #[test]
    fn scalar_mul() {
        assert!(Affine::<BandersnatchConfig>::generator()
            .mul_bigint(0u32)
            .into_affine()
            .is_zero());

        let result: Vec<_> = (1u32..25)
            .map(|k| {
                Affine::<BandersnatchConfig>::generator()
                    .mul_bigint(k)
                    .into_affine()
            })
            .collect();

        let expected = [
            (fp_from_hex!("29C132CC2C0B34C5743711777BBE42F32B79C022AD998465E1E71866A252AE18"), fp_from_hex!("2A6C669EDA123E0F157D8B50BADCD586358CAD81EEE464605E3167B6CC974166")),
            (fp_from_hex!("30433263B93777D7D9AFEF0AD0C2917E183EF5A9DE026EEDA53626C7C6631B2C"), fp_from_hex!("2A2C8F6465887CEEE9EE3185F32B42829E0DFA7F6C65F0071039026018903B8B")),
            (fp_from_hex!("2A7A99B0870A6244304B9231050859771FE941CAD1BCAEDE655D2278621A3466"), fp_from_hex!("2663E58BC157A7CF84D49524700A147BB53489232EA5962C3765BBFE95004080")),
            (fp_from_hex!("158E524E9587DE0F88E5051A8A90301C15743BA1866E17A236C5371967F73EDB"), fp_from_hex!("26699AB9DFDBA66D2FFE63F2D7C7AE0314AFE4E7DCEDAC6DD65A87F58412AD91")),
            (fp_from_hex!("68CBECE0B8FB55450410CBC058928A567EED293D168FAEF44BFDE25F943AABE0"), fp_from_hex!("4E6CC4FE276029F8390F0A114280E0310DBEE412018F03504695B21FDC684238")),
            (fp_from_hex!("5556928265856AF0C775EA91276D9C8094020F3D03B13C429BB015F54CA2344A"), fp_from_hex!("5959EA8C916BFBD5AF302BE4C68D504EEAD4C9974E520A1F87FDF25B08209B74")),
            (fp_from_hex!("300FB01480FA7C2AF7423A9B8DC426F87EC50377D56B162B30D3CF4A22BDE21C"), fp_from_hex!("1D9BECD1CAF90AE527C0E469B4040DBD8B86A69A731586BC9AA45E506CD5A3E5")),
            (fp_from_hex!("E7E3748DB7C5C999A7BCD93D71D671F1F40090423792266F94CB27CA43FCE5C"), fp_from_hex!("563A625521456130DC66F9FD6BDA67330C7BB183B7F2223216C1C9536E1C622F")),
            (fp_from_hex!("6FA853A2BDF39747D61D2037F531CDFECEC506F3DC1BB72A5EA576548629A27C"), fp_from_hex!("46EA6BA9BD77AC92471FAB2253A462653AAB0B2E480B0370B08CB8B8DFA4119A")),
            (fp_from_hex!("3D4674C2547164738ECD3FE23D180A2DDF625449CC93C9EF6D551E12F5BEDA3"), fp_from_hex!("2B31D1DE973C2BD470375DEC5239D6B606033D88135A473B9D41BDC9A729805A")),
            (fp_from_hex!("6F9C2F7DA63E2E9A8617E35994E67FBEAC7692F887DBCA723D6298BC19DEC11C"), fp_from_hex!("72CDC0920C9A864BF9F9ADD455051C2800C289D8BDBA4CBCC29C20DBD6F3EE33")),
            (fp_from_hex!("2190A4FF019CD48811B7AFBF3F6439BAA6FAB5DD569A03900514F99EFA39F969"), fp_from_hex!("611E73584EB4E4527E243EBF4EEBCED0AC81ADC25A4BFEF070652688423C9FC4")),
            (fp_from_hex!("1E0391E1C05728E0F32A6964D0E4A944A303587D11A549E39BE462AFE3656F6C"), fp_from_hex!("63CF210B7EE46C020FB74F2CB87FF37C0E550B99F6B575A5148D96DEBF55876D")),
            (fp_from_hex!("40262A68C869CAF9F6A1B7F58E0FC1EE693B062BC91FECCA4EA25CAC8C6BCC03"), fp_from_hex!("155C375FA20BECCE2E27E58787D8ADB07EFE32542028D0C1B8539D7F800A3913")),
            (fp_from_hex!("56FB7600B5FFD0303A0EC9F2810C170A83D266637DC8E0C27CB7EF9F0436C1D9"), fp_from_hex!("833EB6AB11D4993F346157DDE3E370A6452C19519B37750838F1C2F7370D2CE")),
            (fp_from_hex!("14DDAA48820CB6523B9AE5FE9FE257CBBD1F3D598A28E670A40DA5D1159D864A"), fp_from_hex!("435EA384CEEADB3045B07843AF54D0023B17C49A6EF26F972C8ED00F6E13385F")),
            (fp_from_hex!("3A40BDF40F2881C5D2ABD9992D45BDAC492E2F286EDED1B367649FD047DB5E03"), fp_from_hex!("17F8DC1BCB1FAFC3E11B0B2A8899C08823E822DA0B9ADA3D45618B4A26E72FBD")),
            (fp_from_hex!("4223439F81C281566A72D6BBBDA38A6888C6CAEFE14CC8488A590F7F5AE90131"), fp_from_hex!("42002F019ECAE51DC3ED4A1AD18C1A93A9DA9DC8F5743B80BB03CDD471E632DA")),
            (fp_from_hex!("5C7DBA8147F2160DF10848E03808FD87E5AA0C17A7FCABCB19ADDAB50B5795DC"), fp_from_hex!("65A4BF94A9DCAC5EC8DD242DB39CE1F80C61397A3E6371B066F47581D905FD33")),
            (fp_from_hex!("647E6FEBD4B690BFD29EF3C54317DC6D2F21FC55136BD7210A00729939058604"), fp_from_hex!("99A17BD4E82AC463B09A669BF4E37630A3A0DC3E9CEA2D7C3A63E7014727630")),
            (fp_from_hex!("58001A00662CACB49A9F1467B12EF7F55B846D1B8578EA9EB0725D0FB6C94396"), fp_from_hex!("31A4F4D757DA54965CBA74C01D6743AE8B57DD24153BCEFC9111A4A3A861F26F")),
            (fp_from_hex!("C01D90880A7F58DBD892751E459257E2C70C59F04E2FEAB42FFBBDEBB44CFA5"), fp_from_hex!("1C546FD535BCFC714700A45D588EADABC7C420BEACF07FA40087915D8D7B514E")),
            (fp_from_hex!("3EDB66C32B40ED04100F2A8FF2D5314012778C215E29DECA6E1DDF659CCE69D1"), fp_from_hex!("505E1ECF91775F8269016B3819767C84C75B96F8A90E6E7D2FAF49B9CB3EC4D2")),
            (fp_from_hex!("55172B3682AC5BE8DBC10D464C0146D2AE85E42CC3AFEB5F2D9F90E1E933FD6B"), fp_from_hex!("34C7B4048AC3669EEBA42F95223646711565D17D64078E0BB98D89C1534B4A8D")),
        ];

        for (result, (expected_x, expected_y)) in result.iter().zip(expected) {
            assert!(result.is_on_curve());
            assert_eq!(result.x, expected_x);
            assert_eq!(result.y, expected_y);
        }
    }

    #[test]
    fn point_add() {
        let g = Affine::<BandersnatchConfig>::generator();
        let g_proj: Projective<BandersnatchConfig> = g.into();

        // Test G + G = 2G
        let expected_g2 = Affine::new_unchecked(
            fp_from_hex!("30433263B93777D7D9AFEF0AD0C2917E183EF5A9DE026EEDA53626C7C6631B2C"),
            fp_from_hex!("2A2C8F6465887CEEE9EE3185F32B42829E0DFA7F6C65F0071039026018903B8B"),
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
        let g = Affine::<BandersnatchConfig>::generator();
        let g_proj: Projective<BandersnatchConfig> = g.into();

        // Test G - G = 0
        let zero = g_proj - g_proj;
        assert!(zero.is_zero());

        // Test 2G - G = G
        let g2: Projective<BandersnatchConfig> = Affine::new_unchecked(
            fp_from_hex!("30433263B93777D7D9AFEF0AD0C2917E183EF5A9DE026EEDA53626C7C6631B2C"),
            fp_from_hex!("2A2C8F6465887CEEE9EE3185F32B42829E0DFA7F6C65F0071039026018903B8B"),
        ).into();
        assert_eq!(g2 - g_proj, g_proj);
    }
}
