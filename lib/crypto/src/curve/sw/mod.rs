//! This module contains definitions for the [Short Weierstrass model] of the
//! curve.
//!
//! [Short Weierstrass model]: https://www.hyperelliptic.org/EFD/g1p/auto-shortw.html

use num_traits::Zero;

mod affine;
pub use affine::*;

mod projective;
pub use projective::*;

use crate::{
    bits::BitIteratorBE,
    curve::AffineRepr,
    field::{group::AdditiveGroup, prime::PrimeField},
};

/// Constants and convenience functions that collectively define the
/// [Short Weierstrass model] of the curve.
///
/// In this model, the curve equation is `y² = x³ + a * x + b`, for constants
/// `a` and `b`.
///
/// [Short Weierstrass model]: https://www.hyperelliptic.org/EFD/g1p/auto-shortw.html
pub trait SWCurveConfig: super::CurveConfig {
    /// Coefficient `a` of the curve equation.
    const COEFF_A: Self::BaseField;
    /// Coefficient `b` of the curve equation.
    const COEFF_B: Self::BaseField;
    /// Generator of the prime-order subgroup.
    const GENERATOR: Affine<Self>;

    /// Helper method for computing `elem * Self::COEFF_A`.
    ///
    /// The default implementation should be overridden only if
    /// the product can be computed faster than standard field multiplication
    /// (eg: via doubling if `COEFF_A == 2`, or if `COEFF_A.is_zero()`).
    #[inline(always)]
    fn mul_by_a(elem: Self::BaseField) -> Self::BaseField {
        if Self::COEFF_A.is_zero() {
            Self::BaseField::ZERO
        } else {
            elem * Self::COEFF_A
        }
    }

    /// Helper method for computing `elem + Self::COEFF_B`.
    ///
    /// The default implementation should be overridden only if
    /// the sum can be computed faster than standard field addition (eg: via
    /// doubling).
    #[inline(always)]
    fn add_b(elem: Self::BaseField) -> Self::BaseField {
        if Self::COEFF_B.is_zero() {
            elem
        } else {
            elem + Self::COEFF_B
        }
    }

    /// Check if the provided curve point is in the prime-order subgroup.
    ///
    /// The default implementation multiplies `item` by the order `r` of the
    /// prime-order subgroup, and checks if the result is zero. If the
    /// curve's cofactor is one, this check automatically returns true.
    /// Implementors can choose to override this default impl
    /// if the given curve has faster methods
    /// for performing this check (for example, via leveraging curve
    /// isomorphisms).
    fn is_in_prime_order_subgroup(item: &Affine<Self>) -> bool {
        Self::cofactor_is_one()
            || Self::mul_affine(item, Self::ScalarField::characteristic())
                .is_zero()
    }

    /// Performs cofactor clearing.
    /// The default method is simply to multiply by the cofactor.
    /// Some curves can implement a more efficient algorithm.
    fn clear_cofactor(item: &Affine<Self>) -> Affine<Self> {
        item.mul_by_cofactor()
    }

    /// Default implementation of group multiplication for projective
    /// coordinates.
    fn mul_projective(
        base: &Projective<Self>,
        scalar: impl BitIteratorBE,
    ) -> Projective<Self> {
        sw_double_and_add_projective(base, scalar)
    }

    /// Default implementation of group multiplication for affine
    /// coordinates.
    fn mul_affine(
        base: &Affine<Self>,
        scalar: impl BitIteratorBE,
    ) -> Projective<Self> {
        sw_double_and_add_affine(base, scalar)
    }
}

/// Standard double-and-add method for multiplication by a scalar.
#[inline(always)]
pub fn sw_double_and_add_affine<P: SWCurveConfig>(
    base: &Affine<P>,
    scalar: impl BitIteratorBE,
) -> Projective<P> {
    let mut res = Projective::zero();
    for b in scalar.bit_be_trimmed_iter() {
        res.double_in_place();
        if b {
            res += base;
        }
    }

    res
}

/// Standard double-and-add method for multiplication by a scalar.
#[inline(always)]
pub fn sw_double_and_add_projective<P: SWCurveConfig>(
    base: &Projective<P>,
    scalar: impl BitIteratorBE,
) -> Projective<P> {
    let mut res = Projective::zero();
    for b in scalar.bit_be_trimmed_iter() {
        res.double_in_place();
        if b {
            res += base;
        }
    }

    res
}

#[cfg(test)]
mod test {
    use num_traits::Zero;

    use crate::{
        arithmetic::uint::U256,
        curve::{sw::SWCurveConfig, AffineRepr, CurveConfig, CurveGroup},
        field::{
            fp::{Fp256, FpParams, LIMBS_256},
            group::AdditiveGroup,
        },
        fp_from_hex, fp_from_num, from_num,
    };

    type Affine = super::Affine<Config>;
    type Projective = super::Projective<Config>;

    #[derive(Clone, Default, PartialEq, Eq)]
    struct Config;

    type Fq = Fp256<FqParam>;
    struct FqParam;

    impl FpParams<LIMBS_256> for FqParam {
        const GENERATOR: Fp256<Self> = fp_from_num!("3");
        const MODULUS: U256 = from_num!("115792089237316195423570985008687907853269984665640564039457584007908834671663");
    }

    type Fr = Fp256<FrParam>;
    struct FrParam;

    impl FpParams<LIMBS_256> for FrParam {
        const GENERATOR: Fp256<Self> = fp_from_num!("7");
        const MODULUS: U256 = from_num!("115792089237316195423570985008687907852837564279074904382605163141518161494337");
    }

    impl CurveConfig for Config {
        type BaseField = Fq;
        type ScalarField = Fr;

        const COFACTOR: &'static [u64] = &[0x1, 0x0];
        const COFACTOR_INV: Fr = Fr::ONE;
    }

    impl SWCurveConfig for Config {
        const COEFF_A: Fq = Fq::ZERO;
        const COEFF_B: Fq = fp_from_num!("7");
        const GENERATOR: Affine =
            Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);
    }

    const G_GENERATOR_X: Fq =
        fp_from_num!("55066263022277343669578718895168534326250603453777594175500187360389116729240");

    const G_GENERATOR_Y: Fq =
        fp_from_num!("32670510020758816978083085130507043184471273380659243275938904335757337482424");

    #[test]
    fn scalar_mul() {
        assert!(Affine::generator().mul_bigint(0u32).into_affine().infinity);

        let result: Vec<_> = (1u32..25)
            .map(|k| Affine::generator().mul_bigint(k).into_affine())
            .collect();

        let expected =
            [
                (fp_from_hex!("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"), fp_from_hex!("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8")),
                (fp_from_hex!("C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5"), fp_from_hex!("1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A")),
                (fp_from_hex!("F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9"), fp_from_hex!("388F7B0F632DE8140FE337E62A37F3566500A99934C2231B6CB9FD7584B8E672")),
                (fp_from_hex!("E493DBF1C10D80F3581E4904930B1404CC6C13900EE0758474FA94ABE8C4CD13"), fp_from_hex!("51ED993EA0D455B75642E2098EA51448D967AE33BFBDFE40CFE97BDC47739922")),
                (fp_from_hex!("2F8BDE4D1A07209355B4A7250A5C5128E88B84BDDC619AB7CBA8D569B240EFE4"), fp_from_hex!("D8AC222636E5E3D6D4DBA9DDA6C9C426F788271BAB0D6840DCA87D3AA6AC62D6")),
                (fp_from_hex!("FFF97BD5755EEEA420453A14355235D382F6472F8568A18B2F057A1460297556"), fp_from_hex!("AE12777AACFBB620F3BE96017F45C560DE80F0F6518FE4A03C870C36B075F297")),
                (fp_from_hex!("5CBDF0646E5DB4EAA398F365F2EA7A0E3D419B7E0330E39CE92BDDEDCAC4F9BC"), fp_from_hex!("6AEBCA40BA255960A3178D6D861A54DBA813D0B813FDE7B5A5082628087264DA")),
                (fp_from_hex!("2F01E5E15CCA351DAFF3843FB70F3C2F0A1BDD05E5AF888A67784EF3E10A2A01"), fp_from_hex!("5C4DA8A741539949293D082A132D13B4C2E213D6BA5B7617B5DA2CB76CBDE904")),
                (fp_from_hex!("ACD484E2F0C7F65309AD178A9F559ABDE09796974C57E714C35F110DFC27CCBE"), fp_from_hex!("CC338921B0A7D9FD64380971763B61E9ADD888A4375F8E0F05CC262AC64F9C37")),
                (fp_from_hex!("A0434D9E47F3C86235477C7B1AE6AE5D3442D49B1943C2B752A68E2A47E247C7"), fp_from_hex!("893ABA425419BC27A3B6C7E693A24C696F794C2ED877A1593CBEE53B037368D7")),
                (fp_from_hex!("774AE7F858A9411E5EF4246B70C65AAC5649980BE5C17891BBEC17895DA008CB"), fp_from_hex!("D984A032EB6B5E190243DD56D7B7B365372DB1E2DFF9D6A8301D74C9C953C61B")),
                (fp_from_hex!("D01115D548E7561B15C38F004D734633687CF4419620095BC5B0F47070AFE85A"), fp_from_hex!("A9F34FFDC815E0D7A8B64537E17BD81579238C5DD9A86D526B051B13F4062327")),
                (fp_from_hex!("F28773C2D975288BC7D1D205C3748651B075FBC6610E58CDDEEDDF8F19405AA8"), fp_from_hex!("AB0902E8D880A89758212EB65CDAF473A1A06DA521FA91F29B5CB52DB03ED81")),
                (fp_from_hex!("499FDF9E895E719CFD64E67F07D38E3226AA7B63678949E6E49B241A60E823E4"), fp_from_hex!("CAC2F6C4B54E855190F044E4A7B3D464464279C27A3F95BCC65F40D403A13F5B")),
                (fp_from_hex!("D7924D4F7D43EA965A465AE3095FF41131E5946F3C85F79E44ADBCF8E27E080E"), fp_from_hex!("581E2872A86C72A683842EC228CC6DEFEA40AF2BD896D3A5C504DC9FF6A26B58")),
                (fp_from_hex!("E60FCE93B59E9EC53011AABC21C23E97B2A31369B87A5AE9C44EE89E2A6DEC0A"), fp_from_hex!("F7E3507399E595929DB99F34F57937101296891E44D23F0BE1F32CCE69616821")),
                (fp_from_hex!("DEFDEA4CDB677750A420FEE807EACF21EB9898AE79B9768766E4FAA04A2D4A34"), fp_from_hex!("4211AB0694635168E997B0EAD2A93DAECED1F4A04A95C0F6CFB199F69E56EB77")),
                (fp_from_hex!("5601570CB47F238D2B0286DB4A990FA0F3BA28D1A319F5E7CF55C2A2444DA7CC"), fp_from_hex!("C136C1DC0CBEB930E9E298043589351D81D8E0BC736AE2A1F5192E5E8B061D58")),
                (fp_from_hex!("2B4EA0A797A443D293EF5CFF444F4979F06ACFEBD7E86D277475656138385B6C"), fp_from_hex!("85E89BC037945D93B343083B5A1C86131A01F60C50269763B570C854E5C09B7A")),
                (fp_from_hex!("4CE119C96E2FA357200B559B2F7DD5A5F02D5290AFF74B03F3E471B273211C97"), fp_from_hex!("12BA26DCB10EC1625DA61FA10A844C676162948271D96967450288EE9233DC3A")),
                (fp_from_hex!("352BBF4A4CDD12564F93FA332CE333301D9AD40271F8107181340AEF25BE59D5"), fp_from_hex!("321EB4075348F534D59C18259DDA3E1F4A1B3B2E71B1039C67BD3D8BCF81998C")),
                (fp_from_hex!("421F5FC9A21065445C96FDB91C0C1E2F2431741C72713B4B99DDCB316F31E9FC"), fp_from_hex!("2B90F16D11DABDB616F6DB7E225D1E14743034B37B223115DB20717AD1CD6781")),
                (fp_from_hex!("2FA2104D6B38D11B0230010559879124E42AB8DFEFF5FF29DC9CDADD4ECACC3F"), fp_from_hex!("2DE1068295DD865B64569335BD5DD80181D70ECFC882648423BA76B532B7D67")),
                (fp_from_hex!("FE72C435413D33D48AC09C9161BA8B09683215439D62B7940502BDA8B202E6CE"), fp_from_hex!("6851DE067FF24A68D3AB47E09D72998101DC88E36B4A9D22978ED2FBCF58C5BF")),
            ];

        for (result, (expected_x, expected_y)) in result.iter().zip(expected) {
            assert!(result.is_on_curve());
            assert_eq!(result.x, expected_x);
            assert_eq!(result.y, expected_y);
        }
    }

    #[test]
    fn point_add() {
        let g = Affine::generator();
        let g_proj: Projective = g.into();

        // Test G + G = 2G
        let expected_g2 = Affine::new_unchecked(
            fp_from_hex!("C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5"),
            fp_from_hex!("1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A"),
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
        let g = Affine::generator();
        let g_proj: Projective = g.into();

        // Test G - G = 0
        let zero = g_proj - g_proj;
        assert!(zero.is_zero());

        // Test 2G - G = G
        let g2: Projective = Affine::new_unchecked(
                fp_from_hex!("C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5"),
                fp_from_hex!("1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A"),
            ).into();
        assert_eq!(g2 - g_proj, g_proj);
    }

    #[test]
    fn cofactor_is_one() {
        #[derive(Clone, Default, PartialEq, Eq)]
        struct NotOneCofactorConfig;

        impl CurveConfig for NotOneCofactorConfig {
            type BaseField = Fq;
            type ScalarField = Fr;

            const COFACTOR: &'static [u64] = &[0x0, 0x0, 0x0, 0x1, 0x0];
            const COFACTOR_INV: Fr = Fr::ONE;
        }

        assert!(Config::cofactor_is_one());
        assert!(!NotOneCofactorConfig::cofactor_is_one());
    }
}
