use crate::field::prime::PrimeField;

// TODO#q: remove numbers from docs
// TODO#q: update docs

pub trait PoseidonParams<F: PrimeField> {
    /// State size. (3)
    const T: usize;
    /// SBox degree. (5)
    const D: u8;
    /// Number of full rounds. (8)
    const ROUNDS_F: usize;
    /// Number of partial rounds. (56)
    const ROUNDS_P: usize;
    /// len 3
    const MAT_INTERNAL_DIAG_M_1: &'static [F];
    /// len 64
    const ROUND_CONSTANTS: &'static [&'static [F]];
}
