//! Pedersen hash parameters.

use crate::{curve::CurveConfig, field::prime::PrimeField};

/// Pedersen hash parameters.
pub trait PedersenParams<P: CurveConfig>
where
    <P as CurveConfig>::BaseField: PrimeField,
{
    /// The affine representation type for this Elliptic Curve.
    type AffineRepr;

    /// Number of elements in the hash.
    const N_ELEMENT_BITS_HASH: usize;

    /// Shift point.
    const P_0: Self::AffineRepr;

    /// Constant point -- `P_1`.
    const P_1: Self::AffineRepr;
    /// Constant point -- `P_2`.
    const P_2: Self::AffineRepr;
    /// Constant point -- `P_3`.
    const P_3: Self::AffineRepr;
    /// Constant point -- `P_4`.
    const P_4: Self::AffineRepr;

    /// Low bits of a value to hash.
    const LOW_PART_BITS: u32;
    /// Low part mask for a value to hash.
    const LOW_PART_MASK: <P::BaseField as PrimeField>::BigInt;
}
