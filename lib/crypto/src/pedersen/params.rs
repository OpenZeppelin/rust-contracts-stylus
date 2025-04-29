//! Pedersen hash parameters.

use crate::{
    arithmetic::uint::U256,
    curve::sw::{Affine, Projective, SWCurveConfig},
};
/// Pedersen hash parameters.
pub trait PedersenParams<P: SWCurveConfig> {
    /// Number of elements in the hash.
    const N_ELEMENT_BITS_HASH: usize;
    /// Field prime.
    const FIELD_PRIME: U256;
    
    /// Constant point -- `P_0`. 
    const P_0: Projective<P>;
    /// Constant point -- `P_1`.
    const P_1: Projective<P>;
    /// Constant point -- `P_2`.
    const P_2: Projective<P>;
    /// Constant point -- `P_3`.
    const P_3: Projective<P>;
    
    /// Shift point.
    const SHIFT_POINT: Affine<P>;
}
