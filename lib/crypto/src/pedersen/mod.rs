//! This module contains Pedersen Hash Function implementation.

pub mod instance;
pub mod params;
use alloc::{vec, vec::Vec};

use crate::{
    arithmetic::{uint::U256, BigInteger},
    curve::sw::{Affine, Projective, SWCurveConfig},
    hash::Hasher,
    pedersen::params::PedersenParams,
};

/// Pedersen hash.
// #[derive(Clone, Debug)]
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
    /// Create a new Pedersen hash instance
    pub fn new() -> Self {
        Self {
            params: core::marker::PhantomData,
            curve: core::marker::PhantomData,
            state: vec![],
        }
    }

    /// Add a U256 value to the hash state
    pub fn update(&mut self, value: U256) {
        self.state.push(value);
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
    fn finalize(self) -> Affine<P> {
        let mut point: Projective<P> = F::SHIFT_POINT.into();

        for (idx, value) in self.state.iter().enumerate() {
            let mut element = *value;
            assert!(U256::ZERO <= element && element < F::FIELD_PRIME);

            let start_idx = 2 + idx * F::N_ELEMENT_BITS_HASH;
            let end_idx = 2 + (idx + 1) * F::N_ELEMENT_BITS_HASH;

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

#[cfg(test)]
mod tests {
    use super::*;

    mod starknet {
        use super::*;
        use crate::pedersen::instance::starknet::{
            StarknetCurveConfig, StarknetPedersenParams,
        };

        #[test]
        fn test_pedersen_hash() {
            let pedersen =
                Pedersen::<StarknetPedersenParams, StarknetCurveConfig>::new();
            assert_eq!(pedersen.state.len(), 0);
        }
    }
}
