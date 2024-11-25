//! This module contains a trait with poseidon hash parameters.
//!
//! Consumer of this trait should implement the parameters for the specific
//! poseidon hash instance.
//! Or use the existing instances in the [`crate::poseidon2::instance`] module.

use crate::field::prime::PrimeField;

pub trait PoseidonParams<F: PrimeField> {
    /// State size.
    const T: usize;

    /// SBox degree.
    const D: u8;

    /// Capacity of the sponge construction.
    /// Determines the number of elements not affected directly by input
    /// or not reflected in the output of the sponge hash function.
    const CAPACITY: usize;

    /// Number of full rounds.
    const ROUNDS_F: usize;

    /// Number of partial rounds.
    const ROUNDS_P: usize;

    // TODO#q: we need this parameter just for state size more than 3.
    const MAT_INTERNAL_DIAG_M_1: &'static [F];

    /// The round constants used in the full and partial rounds of the Poseidon
    /// permutation.
    const ROUND_CONSTANTS: &'static [&'static [F]];
}
