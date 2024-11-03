use alloc::{borrow::ToOwned, vec::Vec};

use crate::field::prime::PrimeField;

// pub trait PoseidonParams<F: PrimeField> {
//     const T: usize; // statesize
//     const D: usize; // sbox degree
//     const ROUNDS_F_BEGINNING: usize;
//     const ROUNDS_P: usize;
//     const ROUNDS_F_END: usize;
//     const ROUNDS: usize;
//     const MAT_INTERNAL_DIAG_M_1: &'static [F];
//     const ROUND_CONSTANTS: &'static [&'static [F]];
// }

#[derive(Clone, Debug)]
pub struct Poseidon2Params<F: PrimeField> {
    /// 3
    pub(crate) t: usize, // statesize
    /// 5
    pub(crate) d: usize, // sbox degree
    /// 4
    pub(crate) rounds_f_beginning: usize,
    /// 56
    pub(crate) rounds_p: usize,
    #[allow(dead_code)]
    /// 4
    pub(crate) rounds_f_end: usize,
    /// 64
    pub(crate) rounds: usize,
    /// len 1
    pub(crate) mat_internal_diag_m_1: Vec<F>,
    /// len 64
    pub(crate) round_constants: Vec<Vec<F>>,
}

impl<F: PrimeField> Poseidon2Params<F> {
    #[allow(clippy::too_many_arguments)]

    pub const INIT_SHAKE: &'static str = "Poseidon2";

    // TODO#q: these params should be generically set
    pub fn new(
        t: usize,        // 3
        d: usize,        // 5
        rounds_f: usize, // 8
        rounds_p: usize, // 56
        mat_internal_diag_m_1: &[F],
        round_constants: &[Vec<F>],
    ) -> Self {
        assert!(d == 3 || d == 5 || d == 7 || d == 11);
        assert_eq!(rounds_f % 2, 0);
        let r = rounds_f / 2; // 4
        let rounds = rounds_f + rounds_p; // 64

        Poseidon2Params {
            t,
            d,
            rounds_f_beginning: r,
            rounds_p,
            rounds_f_end: r,
            rounds,
            mat_internal_diag_m_1: mat_internal_diag_m_1.to_owned(),
            round_constants: round_constants.to_owned(),
        }
    }
}
