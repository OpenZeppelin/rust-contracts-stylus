use crate::field::prime::PrimeField;

// TODO#q: remove numbers from docs

pub trait PoseidonParams<F: PrimeField> {
    /// State size. (3)
    const T: usize;

    /// SBox degree. (5)
    const D: u8;

    /// Capacity of the sponge construction.
    /// Determines the number of elements not affected directly by input
    /// or not reflected in the output of the sponge hash function.
    const CAPACITY: usize;

    /// Number of full rounds. (8)
    const ROUNDS_F: usize;

    /// Number of partial rounds. (56)
    const ROUNDS_P: usize;

    // TODO#q: we need this parameter just for state size more than 3.
    /// len 3
    const MAT_INTERNAL_DIAG_M_1: &'static [F];

    /// The round constants used in the full and partial rounds of the Poseidon
    /// permutation. (len 64)
    const ROUND_CONSTANTS: &'static [&'static [F]];
}
