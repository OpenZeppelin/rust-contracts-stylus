//! This module contains the Poseidon hash ([whitepaper]) function implemented
//! as a [Sponge Function].
//!
//! Poseidon permutation here follows referenced in [whitepaper] original [rust
//! implementation] with slight improvements.
//!
//! [Sponge function]: https://en.wikipedia.org/wiki/Sponge_function
//! [whitepaper]: https://eprint.iacr.org/2023/323.pdf
//! [rust implementation]: https://github.com/HorizenLabs/poseidon2

pub mod instance;
pub mod params;

use alloc::{boxed::Box, vec, vec::Vec};

use crate::{field::prime::PrimeField, poseidon2::params::PoseidonParams};

/// Determines whether poseidon sponge in absorbing or squeezing state.
/// In squeezing state, sponge can only squeeze elements.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Absorbing,
    Squeezing,
}

/// Poseidon2 sponge that can absorb any number of [`F`] field elements and be
/// squeezed to a finite number of [`F`] field elements.
#[derive(Clone, Debug)]
pub struct Poseidon2<P: PoseidonParams<F>, F: PrimeField> {
    phantom: core::marker::PhantomData<P>,
    state: Box<[F]>,
    mode: Mode,
    index: usize,
}

impl<P: PoseidonParams<F>, F: PrimeField> Default for Poseidon2<P, F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: PoseidonParams<F>, F: PrimeField> Poseidon2<P, F> {
    /// Create a new Poseidon sponge.
    #[must_use]
    pub fn new() -> Self {
        Self {
            phantom: core::marker::PhantomData,
            state: vec![F::zero(); P::T].into_boxed_slice(),
            mode: Mode::Absorbing,
            // Begin index from `CAPACITY`. Skip capacity elements.
            index: P::CAPACITY,
        }
    }

    /// Size of poseidon sponge's state.
    #[must_use]
    pub const fn state_size() -> usize {
        P::T
    }

    /// Start index of partial rounds.
    #[must_use]
    const fn partial_round_start() -> usize {
        P::ROUNDS_F / 2
    }

    /// End index of partial rounds (noninclusive).
    #[must_use]
    const fn partial_round_end() -> usize {
        Self::partial_round_start() + P::ROUNDS_P
    }

    /// Total number of rounds.
    #[must_use]
    const fn rounds() -> usize {
        P::ROUNDS_F + P::ROUNDS_P
    }

    /// Absorb a single element into the sponge.
    ///
    /// # Panics
    ///
    /// May panic if absorbing while squeezing.
    pub fn absorb(&mut self, elem: &F) {
        if let Mode::Squeezing = self.mode {
            panic!("cannot absorb while squeezing");
        }

        if self.index == Self::state_size() {
            self.permute();
            self.index = P::CAPACITY;
        }

        self.state[self.index] += elem;
        self.index += 1;
    }

    /// Absorb batch of elements into the sponge.
    pub fn absorb_batch(&mut self, elems: &[F]) {
        for elem in elems {
            self.absorb(elem);
        }
    }

    /// Permute elements in the sponge.
    pub fn permute(&mut self) {
        // Linear layer at the beginning.
        self.matmul_external();

        // Run the first half of the full round.
        for round in 0..Self::partial_round_start() {
            self.add_rc_external(round);
            self.apply_sbox_external();
            self.matmul_external();
        }

        // Run the partial round.
        for round in Self::partial_round_start()..Self::partial_round_end() {
            self.add_rc_internal(round);
            self.apply_sbox_internal();
            self.matmul_internal();
        }

        // Run the second half of the full round.
        for round in Self::partial_round_end()..Self::rounds() {
            self.add_rc_external(round);
            self.apply_sbox_external();
            self.matmul_external();
        }
    }

    /// Squeeze a single element from the sponge.
    pub fn squeeze(&mut self) -> F {
        if self.mode == Mode::Absorbing || self.index == Self::state_size() {
            self.permute();
            self.mode = Mode::Squeezing;
            self.index = P::CAPACITY;
        }

        let elem = self.state[self.index];
        self.index += 1;
        elem
    }

    /// Squeeze a batch of elements from the sponge.
    pub fn squeeze_batch(&mut self, n: usize) -> Vec<F> {
        (0..n).map(|_| self.squeeze()).collect()
    }

    /// Apply sbox to the entire state in the external round.
    fn apply_sbox_external(&mut self) {
        for elem in &mut self.state {
            *elem = elem.pow(P::D);
        }
    }

    /// Apply sbox to the first element in the internal round.
    fn apply_sbox_internal(&mut self) {
        self.state[0] = self.state[0].pow(P::D);
    }

    /// Apply the external MDS matrix `M_E` to the state.
    #[allow(clippy::needless_range_loop)]
    fn matmul_external(&mut self) {
        let t = Self::state_size();
        match t {
            2 => {
                // Matrix circ(2, 1)
                let sum = self.state[0] + self.state[1];
                self.state[0] += sum;
                self.state[1] += sum;
            }
            3 => {
                // Matrix circ(2, 1, 1).
                let sum = self.state[0] + self.state[1] + self.state[2];
                self.state[0] += sum;
                self.state[1] += sum;
                self.state[2] += sum;
            }
            4 => {
                self.matmul_m4();
            }
            8 | 12 | 16 | 20 | 24 => {
                self.matmul_m4();

                // Applying second cheap matrix for t > 4.
                let t4 = t / 4;
                let mut stored = [F::zero(); 4];
                for l in 0..4 {
                    stored[l] = self.state[l];
                    for j in 1..t4 {
                        stored[l] += &self.state[4 * j + l];
                    }
                }
                for i in 0..self.state.len() {
                    self.state[i] += &stored[i % 4];
                }
            }
            _ => {
                panic!("not supported state size")
            }
        }
    }

    /// Apply the cheap 4x4 MDS matrix to each 4-element part of the state.
    fn matmul_m4(&mut self) {
        let state = &mut self.state;
        let t = Self::state_size();
        let t4 = t / 4;
        for i in 0..t4 {
            let start_index = i * 4;
            let mut t_0 = state[start_index];
            t_0 += &state[start_index + 1];
            let mut t_1 = state[start_index + 2];
            t_1 += &state[start_index + 3];
            let mut t_2 = state[start_index + 1];
            t_2.double_in_place();
            t_2 += &t_1;
            let mut t_3 = state[start_index + 3];
            t_3.double_in_place();
            t_3 += &t_0;
            let mut t_4 = t_1;
            t_4.double_in_place();
            t_4.double_in_place();
            t_4 += &t_3;
            let mut t_5 = t_0;
            t_5.double_in_place();
            t_5.double_in_place();
            t_5 += &t_2;
            let mut t_6 = t_3;
            t_6 += &t_5;
            let mut t_7 = t_2;
            t_7 += &t_4;
            state[start_index] = t_6;
            state[start_index + 1] = t_5;
            state[start_index + 2] = t_7;
            state[start_index + 3] = t_4;
        }
    }

    /// Apply the internal MDS matrix `M_I` to the state.
    fn matmul_internal(&mut self) {
        let t = Self::state_size();

        match t {
            2 => {
                // [2, 1]
                // [1, 3]
                let sum = self.state[0] + self.state[1];
                self.state[0] += &sum;
                self.state[1].double_in_place();
                self.state[1] += &sum;
            }
            3 => {
                // [2, 1, 1]
                // [1, 2, 1]
                // [1, 1, 3]
                let sum = self.state[0] + self.state[1] + self.state[2];
                self.state[0] += &sum;
                self.state[1] += &sum;
                self.state[2].double_in_place();
                self.state[2] += &sum;
            }
            4 | 8 | 12 | 16 | 20 | 24 => {
                let sum = self.state.iter().sum();

                // Add sum + diag entry * element to each element.
                for i in 0..self.state.len() {
                    self.state[i] *= &P::MAT_INTERNAL_DIAG_M_1[i];
                    self.state[i] += &sum;
                }
            }
            _ => {
                panic!("not supported state size")
            }
        }
    }

    /// Add a round constant to the entire state in external round.
    fn add_rc_external(&mut self, round: usize) {
        for (a, b) in
            self.state.iter_mut().zip(P::ROUND_CONSTANTS[round].iter())
        {
            *a += b;
        }
    }

    // Add a round constant to the first state element in internal round.
    fn add_rc_internal(&mut self, round: usize) {
        self.state[0] += P::ROUND_CONSTANTS[round][0];
    }
}
