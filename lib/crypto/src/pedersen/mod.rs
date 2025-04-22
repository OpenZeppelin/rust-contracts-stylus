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
    hash::Hasher,
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

    /// Add `input` values to the hash state.
    pub fn update(&mut self, input: &[U256]) {
        self.state.extend(input);
    }

    /// Hashes the input values and returns the result as `x` coordinate of
    /// the point on the curve.
    ///
    /// # Panics
    ///
    /// * If [`Pedersen::finalize`] panics.
    pub fn hash(mut self, input: &[U256]) -> P::BaseField {
        self.update(input);
        let hash = self.finalize();
        hash.x().expect("Pedersen hash failed")
    }
}

impl<F: PedersenParams<P>, P: SWCurveConfig> Hasher for Pedersen<F, P> {
    type Output = Affine<P>;

    /// Update the hash state with a new element.
    fn update(&mut self, input: impl AsRef<[u8]>) {
        let value = U256::from_bytes_le(input.as_ref());
        self.state.push(value);
    }

    /// Finalize the hash and return the result.
    ///
    /// # Panics
    ///
    /// * If one of the input values is higher than
    ///   [`PedersenParams::FIELD_PRIME`].
    /// * If the input values contains more elements than length of
    ///   [`PedersenParams::CONSTANT_POINTS`] /
    ///   [`PedersenParams::N_ELEMENT_BITS_HASH`].
    fn finalize(self) -> Affine<P> {
        let mut point: Projective<P> = F::SHIFT_POINT.into();

        for (idx, value) in self.state.iter().enumerate() {
            let mut element = *value;
            assert!(
                U256::ZERO <= element && element < F::FIELD_PRIME,
                "Pedersen hash failed -- invalid input"
            );

            let start_idx = 2 + idx * F::N_ELEMENT_BITS_HASH;
            let end_idx = 2 + (idx + 1) * F::N_ELEMENT_BITS_HASH;

            if end_idx > F::CONSTANT_POINTS.len() {
                panic!("Pedersen hash failed -- too many elements");
            }

            let point_list = &F::CONSTANT_POINTS[start_idx..end_idx];

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
