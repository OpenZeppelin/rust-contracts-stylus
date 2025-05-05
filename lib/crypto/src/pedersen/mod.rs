//! This module contains Pedersen Hash Function implementation.
///
/// Based on the [Starknet] implementation of the Pedersen Hash Function.
///
/// [Starknet]: <https://github.com/starkware-libs/cairo-lang/blob/master/src/starkware/crypto/signature/fast_pedersen_hash.py>
pub mod instance;
pub mod params;

use crate::{
    arithmetic::{uint::U256, BigInteger},
    curve::{
        sw::{Affine, Projective, SWCurveConfig},
        AffineRepr, PrimeGroup,
    },
    from_num,
    pedersen::params::PedersenParams,
};

/// Low part bits.
const LOW_PART_BITS: u32 = 248;
/// Low part mask. (2**248 - 1)
const LOW_PART_MASK: U256 = from_num!(
    "452312848583266388373324160190187140051835877600158453279131187530910662655"
);
/// Pedersen hash.
#[derive(Clone, Debug)]
pub struct Pedersen<F: PedersenParams<P>, P: SWCurveConfig> {
    params: core::marker::PhantomData<F>,
    curve: core::marker::PhantomData<P>,
}

impl<F: PedersenParams<P>, P: SWCurveConfig> Default for Pedersen<F, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: PedersenParams<P>, P: SWCurveConfig> Pedersen<F, P> {
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
        element: U256,
        p1: Projective<P>,
        p2: Projective<P>,
    ) -> Projective<P> {
        assert!(
            U256::ZERO <= element && element < F::FIELD_PRIME,
            "Element integer value is out of range"
        );

        let high_nibble = element >> LOW_PART_BITS;
        let low_part = element & LOW_PART_MASK;
        p1.mul_bigint(low_part) + p2.mul_bigint(high_nibble)
    }

    /// Computes the Starkware version of the Pedersen hash of x and y.
    ///
    /// The hash is defined by:
    /// [`F::SHIFT_POINT`] + `x_low` * [`F::P_0`] + `x_high` * [`F::P_1`] +
    /// `y_low` * [`F::P_2`] + `y_high` * [`F::P_3`]
    ///
    /// where `x_low` is the 248 low bits of `x`, `x_high` is the 4 high bits of
    /// `x` and similarly for `y`. [`F::SHIFT_POINT`], [`F::P_0`],
    /// [`F::P_1`], [`F::P_2`], [`F::P_3`] are constant points generated
    /// from the digits of pi.
    ///
    /// # Arguments
    ///
    /// * `&self` - Pedersen hasher instance.
    /// * `x` - The x coordinate of the point to hash.
    /// * `y` - The y coordinate of the point to hash.
    #[must_use]
    pub fn hash(&self, x: U256, y: U256) -> Option<P::BaseField> {
        let hash: Projective<P> = F::SHIFT_POINT
            + Self::process_single_element(x, F::P_0.into(), F::P_1.into())
            + Self::process_single_element(y, F::P_2.into(), F::P_3.into());

        let hash: Affine<P> = hash.into();
        hash.x()
    }
}
