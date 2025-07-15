//! This module contains Pedersen Hash Function implementation.
///
/// Based on the [Starknet] implementation of the Pedersen Hash Function.
///
/// [Starknet]: <https://github.com/starkware-libs/cairo-lang/blob/master/src/starkware/crypto/signature/fast_pedersen_hash.py>
pub mod instance;
pub mod params;

use crate::{
    curve::{AffineRepr, CurveConfig},
    field::prime::PrimeField,
    pedersen::params::PedersenParams,
};

/// Pedersen hash.
#[derive(Clone, Debug)]
pub struct Pedersen<F: PedersenParams<P>, P: CurveConfig>
where
    <P as CurveConfig>::BaseField: PrimeField,
    F::AffineRepr: AffineRepr<
        Config = P,
        BaseField = P::BaseField,
        ScalarField = P::ScalarField,
    >,
{
    params: core::marker::PhantomData<F>,
    curve: core::marker::PhantomData<P>,
}

impl<F: PedersenParams<P>, P: CurveConfig> Default for Pedersen<F, P>
where
    <P as CurveConfig>::BaseField: PrimeField,
    F::AffineRepr: AffineRepr<
        Config = P,
        BaseField = P::BaseField,
        ScalarField = P::ScalarField,
    >,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<F: PedersenParams<P>, P: CurveConfig> Pedersen<F, P>
where
    <P as CurveConfig>::BaseField: PrimeField,
    F::AffineRepr: AffineRepr<
        Config = P,
        BaseField = P::BaseField,
        ScalarField = P::ScalarField,
    >,
{
    #[must_use]
    #[inline]
    /// Creates a new Pedersen hash instance.
    pub fn new() -> Self {
        Self {
            params: core::marker::PhantomData,
            curve: core::marker::PhantomData,
        }
    }

    fn process_single_element(
        element: P::BaseField,
        p1: F::AffineRepr,
        p2: F::AffineRepr,
    ) -> <F::AffineRepr as AffineRepr>::Group {
        let element = element.into_bigint();

        let high_nibble = element >> F::LOW_PART_BITS;
        let low_part = element & F::LOW_PART_MASK;

        p1.mul_bigint(low_part) + p2.mul_bigint(high_nibble)
    }

    /// Computes the Starkware version of the Pedersen hash of x and y.
    ///
    /// The hash is defined by:
    /// [`PedersenParams::P_0`] + `x_low` * [`PedersenParams::P_1`] +
    /// `x_high` * [`PedersenParams::P_2`] + `y_low` * [`PedersenParams::P_3`] +
    /// `y_high` * [`PedersenParams::P_4`]
    ///
    /// where `x_low` is the 248 low bits of `x`, `x_high` is the 4 high bits of
    /// `x` and similarly for `y`. [`PedersenParams::P_0`],
    /// [`PedersenParams::P_1`], [`PedersenParams::P_2`],
    /// [`PedersenParams::P_3`], [`PedersenParams::P_4`] are constant points
    /// generated from the digits of pi.
    ///
    /// # Arguments
    ///
    /// * `&self` - Pedersen hasher instance.
    /// * `x` - The x coordinate of the point to hash.
    /// * `y` - The y coordinate of the point to hash.
    #[must_use]
    pub fn hash(
        &self,
        x: P::BaseField,
        y: P::BaseField,
    ) -> Option<P::BaseField> {
        let hash: <F::AffineRepr as AffineRepr>::Group = F::P_0.into_group()
            + Self::process_single_element(x, F::P_1, F::P_2)
            + Self::process_single_element(y, F::P_3, F::P_4);

        let hash: F::AffineRepr = hash.into();
        hash.x()
    }
}
