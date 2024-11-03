use crate::field::prime::PrimeField;

pub trait PoseidonParams<F: PrimeField> {
    /// 3
    const T: usize; // statesize
    /// 5
    const D: u8; // sbox degree
    /// 8
    const ROUNDS_F: usize;
    /// 4
    const ROUNDS_F_BEGINNING: usize = Self::ROUNDS_F / 2;
    /// 56
    const ROUNDS_P: usize;
    /// 4
    const ROUNDS_F_END: usize = Self::ROUNDS_F / 2;
    /// 64
    const ROUNDS: usize = Self::ROUNDS_F + Self::ROUNDS_P;
    /// len 1
    const MAT_INTERNAL_DIAG_M_1: &'static [F];
    /// len 64
    const ROUND_CONSTANTS: &'static [&'static [F]];

    fn round_f_beginning() -> usize {
        Self::ROUNDS_F / 2
    }

    fn round_f_end() -> usize {
        Self::ROUNDS_F / 2
    }

    fn rounds() -> usize {
        Self::ROUNDS_F + Self::ROUNDS_P
    }
}
