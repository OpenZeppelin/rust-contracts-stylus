use bn::U256;

use core::fmt::Debug;

/// Elliptic curve.
///
/// This trait is intended to be impl'd by a ZST which represents a concrete
/// elliptic curve.
///
/// Other traits in this crate which are bounded by [`Curve`] are intended to
/// be impl'd by these ZSTs.
pub trait Curve:
    'static + Copy + Clone + Debug + Default + Eq + Ord + Send + Sync
{
    /// Order of this elliptic curve, i.e. number of elements in the scalar
    /// field.
    const ORDER: U256;
}

/// Parameters for elliptic curves of prime order which can be described by the
/// short Weierstrass equation.
pub trait PrimeCurveParams {
    /// Coefficient `a` in the curve equation.
    const EQUATION_A: U256;
    /// Coefficient `b` in the curve equation.
    const EQUATION_B: U256;
    /// Generator point's affine coordinates: (x, y).
    const GENERATOR: (U256, U256);
}
