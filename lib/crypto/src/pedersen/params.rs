//! Pedersen hash parameters.

use crate::{
    arithmetic::uint::U256,
    curve::sw::{Affine, SWCurveConfig},
};
/// Pedersen hash parameters.
pub trait PedersenParams<P: SWCurveConfig> {
    /// Number of elements in the hash.
    const N_ELEMENT_BITS_HASH: usize;
    /// Field prime.
    const FIELD_PRIME: U256;
    /// Constant points.
    const CONSTANT_POINTS: &'static [Affine<P>];
    /// Shift point.
    const SHIFT_POINT: Affine<P>;
}
