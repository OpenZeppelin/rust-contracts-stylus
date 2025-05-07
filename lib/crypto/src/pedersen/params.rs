//! Pedersen hash parameters.

use crate::{
    curve::{
        sw::{Affine, SWCurveConfig},
        CurveConfig,
    },
    field::prime::PrimeField,
};

/// Pedersen hash parameters.
pub trait PedersenParams<P: SWCurveConfig>
where
    <P as CurveConfig>::BaseField: PrimeField,
{
    /// Number of elements in the hash.
    const N_ELEMENT_BITS_HASH: usize;

    /// Shift point.
    const P_0: Affine<P>;

    /// Constant point -- `P_1`.
    const P_1: Affine<P>;
    /// Constant point -- `P_2`.
    const P_2: Affine<P>;
    /// Constant point -- `P_3`.
    const P_3: Affine<P>;
    /// Constant point -- `P_4`.
    const P_4: Affine<P>;

    /// Low bits of a value to hash.
    const LOW_PART_BITS: u32;
    /// Low part mask for a value to hash.
    const LOW_PART_MASK: <P::BaseField as PrimeField>::BigInt;
}
