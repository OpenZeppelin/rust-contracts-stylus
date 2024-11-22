pub mod instance;
pub mod params;

use alloc::vec::Vec;

use crate::{field::prime::PrimeField, poseidon2::params::PoseidonParams};

#[derive(Clone, Copy, Debug)]
enum Mode {
    Absorbing,
    Squeezing,
}

#[derive(Clone, Debug)]
pub struct Poseidon2<P: PoseidonParams<F>, F: PrimeField> {
    phantom: core::marker::PhantomData<P>,
    state: Box<[F]>,
    mode: Mode,
    index: usize,
}

impl<P: PoseidonParams<F>, F: PrimeField> Poseidon2<P, F> {
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

    #[must_use]
    pub const fn state_size(&self) -> usize {
        P::T
    }

    #[must_use]
    const fn round_f_beginning(&self) -> usize {
        P::ROUNDS_F / 2
    }

    #[must_use]
    const fn round_f_end(&self) -> usize {
        P::ROUNDS_F / 2
    }

    #[must_use]
    const fn rounds(&self) -> usize {
        P::ROUNDS_F + P::ROUNDS_P
    }

    /// Absorb a single element into the sponge.
    pub fn absorb(&mut self, elem: &F) {
        if let Mode::Squeezing = self.mode {
            panic!("cannot absorb while squeezing");
        }

        if self.index == self.state_size() {
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

        for r in 0..self.round_f_beginning() {
            self.add_rc(P::ROUND_CONSTANTS[r]);
            self.sbox();
            self.matmul_external();
        }

        let p_end = self.round_f_beginning() + P::ROUNDS_P;
        for r in self.round_f_beginning()..p_end {
            self.state[0] += P::ROUND_CONSTANTS[r][0];
            self.state[0] = Self::sbox_p(&self.state[0]);
            self.matmul_internal();
        }

        for r in p_end..self.rounds() {
            self.add_rc(P::ROUND_CONSTANTS[r]);
            self.sbox();
            self.matmul_external();
        }
    }

    /// Squeeze a single element from the sponge.
    pub fn squeeze(&mut self) -> F {
        if matches!(self.mode, Mode::Absorbing)
            || self.index == self.state_size()
        {
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

    // TODO#q: rename to external sbox
    fn sbox(&mut self) {
        for elem in self.state.iter_mut() {
            *elem = Self::sbox_p(elem);
        }
    }

    // TODO#q: rename to internal sbox
    fn sbox_p(input: &F) -> F {
        input.pow(P::D)
    }

    fn matmul_m4(&mut self) {
        let state = &mut self.state;
        let t = P::T;
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

    /// Apply the external MDS matrix M_E to the state.
    fn matmul_external(&mut self) {
        let t = P::T;
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
                // Applying cheap 4x4 MDS matrix to each 4-element part of the
                // state.
                self.matmul_m4();
            }
            8 | 12 | 16 | 20 | 24 => {
                // Applying cheap 4x4 MDS matrix to each 4-element part of the
                // state.
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
                panic!()
            }
        }
    }

    /// Apply the internal MDS matrix M_I to the state.
    fn matmul_internal(&mut self) {
        let t = P::T;

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
                panic!()
            }
        }
    }

    /// Add a round constant to the state.
    fn add_rc(&mut self, rc: &[F]) {
        for (a, b) in self.state.iter_mut().zip(rc.iter()) {
            *a += b;
        }
    }
}

#[allow(unused_imports)]
#[cfg(test)]
pub fn random_scalar<F: PrimeField + crypto_bigint::Random>() -> F {
    let mut rng = rand::thread_rng();
    F::random(&mut rng)
}
