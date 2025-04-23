//! This module contains Pedersen Hash Function implementation.

pub mod instance;
pub mod params;
use alloc::{vec, vec::Vec};

use crate::{
    arithmetic::{uint::U256, BigInteger},
    curve::{
        sw::{Affine, Projective, SWCurveConfig},
        AffineRepr,
    },
    pedersen::params::PedersenParams,
};

/// Pedersen hash.
#[derive(Clone, Debug)]
pub struct Pedersen<F: PedersenParams<P>, P: SWCurveConfig> {
    params: core::marker::PhantomData<F>,
    curve: core::marker::PhantomData<P>,
    state: Vec<U256>,
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
            state: vec![],
        }
    }

    /// Hashes the input values and returns the result as `x` coordinate of
    /// the point on the curve.
    ///
    /// # Arguments
    ///
    /// * `mut self` - Pedersen hasher instance.
    /// * `input` - The input values to hash.
    ///
    /// # Panics
    ///
    /// * If [`Pedersen::finalize`] panics.
    #[must_use]
    pub fn hash(mut self, input: &[U256]) -> P::BaseField {
        self.update(input);
        let hash = self.finalize();
        hash.x().expect("Pedersen hash failed")
    }

    /// Add `input` values to the hash state.
    ///
    /// # Arguments
    ///
    /// * `mut self` - Mutable reference to the Pedersen hasher instance.
    /// * `input` - The input values to update the hasher state.
    pub fn update(&mut self, input: &[U256]) {
        self.state.extend(input);
    }

    /// Finalize the hash and return the result.
    ///
    /// # Arguments
    ///
    /// * `self` - Pedersen hasher instance.
    ///
    /// # Panics
    ///
    /// * If one of the input values is higher than
    ///   [`PedersenParams::FIELD_PRIME`].
    /// * If the input values contains more elements than length of
    ///   [`PedersenParams::CONSTANT_POINTS`] /
    ///   [`PedersenParams::N_ELEMENT_BITS_HASH`].
    pub fn finalize(self) -> Affine<P> {
        let mut point: Projective<P> = F::SHIFT_POINT.into();

        let constant_points = F::constant_points();
        let constant_points_len = constant_points.len();

        for (idx, value) in self.state.iter().enumerate() {
            let mut element = *value;
            assert!(
                U256::ZERO <= element && element < F::FIELD_PRIME,
                "Pedersen hash failed -- invalid input"
            );

            let start_idx = 2 + idx * F::N_ELEMENT_BITS_HASH;
            let end_idx = 2 + (idx + 1) * F::N_ELEMENT_BITS_HASH;

            assert!(
                end_idx <= constant_points_len,
                "Pedersen hash failed -- too many elements"
            );

            let point_list = &constant_points[start_idx..end_idx];

            assert!(point_list.len() == F::N_ELEMENT_BITS_HASH);

            for pt in point_list {
                assert!(pt.x != point.x, "Unhashable input.");
                if element.ct_is_odd() {
                    point += pt;
                }
                element.div2_assign();
            }
            assert!(element.is_zero());
        }

        // Convert to Affine coordinates.
        point.into()
    }
}
