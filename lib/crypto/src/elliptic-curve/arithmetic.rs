//! Point arithmetic implementation optimised for different curve equations.
//!
//! Support for formulas specialized to the short Weierstrass equation's
//! 𝒂-coefficient.
//!
//! Algorithms from [Renes-Costello-Batina 2015]: https://eprint.iacr.org/2015/1060

use super::{
    affine::AffinePoint, curve::PrimeCurveParams, field::Field, p256::P256,
    projective::ProjectivePoint,
};

/// Elliptic point arithmetic implementation
///
/// Provides implementation of point arithmetic (point addition, point doubling)
/// which might be optimized for the curve.
pub trait PointArithmetic<C: PrimeCurveParams> {
    /// Returns `lhs + rhs`.
    fn add(
        lhs: &ProjectivePoint<C>,
        rhs: &ProjectivePoint<C>,
    ) -> ProjectivePoint<C>;

    /// Returns `lhs + rhs`.
    fn add_mixed(
        lhs: &ProjectivePoint<C>,
        rhs: &AffinePoint<C>,
    ) -> ProjectivePoint<C>;

    /// Returns `point + point`.
    fn double(point: &ProjectivePoint<C>) -> ProjectivePoint<C>;
}

/// The 𝒂-coefficient of the short Weierstrass equation is -3.
impl<C: PrimeCurveParams> PointArithmetic<C> for P256 {
    /// Implements complete addition for curves with `a = -3`
    ///
    /// Implements the complete addition formula from [Renes-Costello-Batina
    /// 2015] (Algorithm 4). The comments after each line indicate which
    /// algorithm steps are being performed.
    ///
    /// [Renes-Costello-Batina 2015]: https://eprint.iacr.org/2015/1060
    fn add(
        lhs: &ProjectivePoint<C>,
        rhs: &ProjectivePoint<C>,
    ) -> ProjectivePoint<C> {
        let xx = lhs.x * rhs.x; // 1
        let yy = lhs.y * rhs.y; // 2
        let zz = lhs.z * rhs.z; // 3
        let xy_pairs = ((lhs.x + lhs.y) * (rhs.x + rhs.y)) - (xx + yy); // 4, 5, 6, 7, 8
        let yz_pairs = ((lhs.y + lhs.z) * (rhs.y + rhs.z)) - (yy + zz); // 9, 10, 11, 12, 13
        let xz_pairs = ((lhs.x + lhs.z) * (rhs.x + rhs.z)) - (xx + zz); // 14, 15, 16, 17, 18

        let bzz_part = xz_pairs - (C::EQUATION_B * zz); // 19, 20
        let bzz3_part = bzz_part.double() + bzz_part; // 21, 22
        let yy_m_bzz3 = yy - bzz3_part; // 23
        let yy_p_bzz3 = yy + bzz3_part; // 24

        let zz3 = zz.double() + zz; // 26, 27
        let bxz_part = (C::EQUATION_B * xz_pairs) - (zz3 + xx); // 25, 28, 29
        let bxz3_part = bxz_part.double() + bxz_part; // 30, 31
        let xx3_m_zz3 = xx.double() + xx - zz3; // 32, 33, 34

        let x = (yy_p_bzz3 * xy_pairs) - (yz_pairs * bxz3_part); // 35, 39, 40
        let y = (yy_p_bzz3 * yy_m_bzz3) + (xx3_m_zz3 * bxz3_part); // 36, 37, 38
        let z = (yy_m_bzz3 * yz_pairs) + (xy_pairs * xx3_m_zz3); // 41, 42, 43
        ProjectivePoint { x, y, z }
    }

    /// Implements complete mixed addition for curves with `a = -3`.
    ///
    /// Implements the complete mixed addition formula from
    /// [Renes-Costello-Batina 2015] (Algorithm 5). The comments after each
    /// line indicate which algorithm steps are being performed.
    ///
    /// [Renes-Costello-Batina 2015]: https://eprint.iacr.org/2015/1060
    fn add_mixed(
        lhs: &ProjectivePoint<C>,
        rhs: &AffinePoint<C>,
    ) -> ProjectivePoint<C> {
        let xx = lhs.x * rhs.x; // 1
        let yy = lhs.y * rhs.y; // 2
        let xy_pairs = ((lhs.x + lhs.y) * (rhs.x + rhs.y)) - (xx + yy); // 3, 4, 5, 6, 7
        let yz_pairs = (rhs.y * lhs.z) + lhs.y; // 8, 9 (t4)
        let xz_pairs = (rhs.x * lhs.z) + lhs.x; // 10, 11 (y3)

        let bz_part = xz_pairs - (C::EQUATION_B * lhs.z); // 12, 13
        let bz3_part = bz_part.double() + bz_part; // 14, 15
        let yy_m_bzz3 = yy - bz3_part; // 16
        let yy_p_bzz3 = yy + bz3_part; // 17

        let z3 = lhs.z.double() + lhs.z; // 19, 20
        let bxz_part = (C::EQUATION_B * xz_pairs) - (z3 + xx); // 18, 21, 22
        let bxz3_part = bxz_part.double() + bxz_part; // 23, 24
        let xx3_m_zz3 = xx.double() + xx - z3; // 25, 26, 27

        let x = (yy_p_bzz3 * xy_pairs) - (yz_pairs * bxz3_part); // 28, 32, 33
        let y = (yy_p_bzz3 * yy_m_bzz3) + (xx3_m_zz3 * bxz3_part); // 29, 30, 31
        let z = (yy_m_bzz3 * yz_pairs) + (xy_pairs * xx3_m_zz3); // 34, 35, 36
        ProjectivePoint { x, y, z }
    }

    /// Implements point doubling for curves with `a = -3`
    ///
    /// Implements the exception-free point doubling formula from
    /// [Renes-Costello-Batina 2015] (Algorithm 6). The comments after each
    /// line indicate which algorithm steps are being performed.
    ///
    /// [Renes-Costello-Batina 2015]: https://eprint.iacr.org/2015/1060
    fn double(point: &ProjectivePoint<C>) -> ProjectivePoint<C> {
        let xx = point.x * point.x; // 1
        let yy = point.y * point.y; // 2
        let zz = point.z * point.z; // 3
        let xy2 = (point.x * point.y).double(); // 4, 5
        let xz2 = (point.x * point.z).double(); // 6, 7

        let bzz_part = (C::EQUATION_B * zz) - xz2; // 8, 9
        let bzz3_part = bzz_part.double() + bzz_part; // 10, 11
        let yy_m_bzz3 = yy - bzz3_part; // 12
        let yy_p_bzz3 = yy + bzz3_part; // 13
        let y_frag = yy_p_bzz3 * yy_m_bzz3; // 14
        let x_frag = yy_m_bzz3 * xy2; // 15

        let zz3 = zz.double() + zz; // 16, 17
        let bxz2_part = (C::EQUATION_B * xz2) - (zz3 + xx); // 18, 19, 20
        let bxz6_part = bxz2_part.double() + bxz2_part; // 21, 22
        let xx3_m_zz3 = xx.double() + xx - zz3; // 23, 24, 25

        let y = y_frag + (xx3_m_zz3 * bxz6_part); // 26, 27
        let yz2 = (point.y * point.z).double(); // 28, 29
        let x = x_frag - (bxz6_part * yz2); // 30, 31
        let z = (yz2 * yy).double().double(); // 32, 33, 34

        ProjectivePoint { x, y, z }
    }
}
