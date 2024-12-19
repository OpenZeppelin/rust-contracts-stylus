//! This module contains a trait with poseidon hash parameters.
//!
//! Consumer of this trait should implement the parameters for the specific
//! poseidon hash instance.
//! Or use the existing instances in the [`crate::poseidon2::instance`] module.

use crate::field::prime::PrimeField;

/// Poseidon hash parameters.
pub trait PoseidonParams<F: PrimeField> {
    /// State size.
    const T: usize;

    /// Sbox degree.
    const D: u8;

    /// Capacity of the sponge construction.
    /// Determines the number of elements not affected directly by input
    /// or not reflected in the output of the sponge hash function.
    const CAPACITY: usize;

    /// Number of full rounds.
    const ROUNDS_F: usize;

    /// Number of partial rounds.
    const ROUNDS_P: usize;

    /// MDS (Maximum Distance Separable) matrix used in the Poseidon
    /// permutation.
    const MAT_INTERNAL_DIAG_M_1: &'static [F];

    /// The round constants used in the full and partial rounds of the Poseidon
    /// permutation.
    const ROUND_CONSTANTS: &'static [&'static [F]];
}
