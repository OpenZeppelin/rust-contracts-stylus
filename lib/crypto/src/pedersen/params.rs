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

    /// Constant point -- `P_0`.
    const P_0: Affine<P>;
    /// Constant point -- `P_1`.
    const P_1: Affine<P>;
    /// Constant point -- `P_2`.
    const P_2: Affine<P>;
    /// Constant point -- `P_3`.
    const P_3: Affine<P>;

    /// Shift point.
    const SHIFT_POINT: Affine<P>;
}
