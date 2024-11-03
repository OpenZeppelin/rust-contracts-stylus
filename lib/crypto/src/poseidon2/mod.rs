pub mod instance;
pub mod params;

use alloc::{borrow::ToOwned, vec::Vec};

use crate::{field::prime::PrimeField, poseidon2::params::PoseidonParams};

#[derive(Clone, Debug)]
pub struct Poseidon2<P: PoseidonParams<F>, F: PrimeField> {
    phantom: core::marker::PhantomData<(P, F)>,
}

impl<P: PoseidonParams<F>, F: PrimeField> Poseidon2<P, F> {
    #[must_use]
    pub fn new() -> Self {
        Self { phantom: core::marker::PhantomData }
    }

    #[must_use]
    pub fn get_t(&self) -> usize {
        P::T
    }

    pub fn permutation(&self, input: &[F]) -> Vec<F> {
        let t = P::T;
        assert_eq!(input.len(), t);

        let mut current_state = input.to_owned();

        // Linear layer at beginning
        self.matmul_external(&mut current_state);

        for r in 0..P::ROUNDS_F_BEGINNING {
            current_state = self.add_rc(&current_state, &P::ROUND_CONSTANTS[r]);
            current_state = self.sbox(&current_state);
            self.matmul_external(&mut current_state);
        }

        let p_end = P::ROUNDS_F_BEGINNING + P::ROUNDS_P;
        for r in P::ROUNDS_F_BEGINNING..p_end {
            current_state[0].add_assign(P::ROUND_CONSTANTS[r][0]);
            current_state[0] = self.sbox_p(&current_state[0]);
            self.matmul_internal(&mut current_state, P::MAT_INTERNAL_DIAG_M_1);
        }

        for r in p_end..P::ROUNDS {
            current_state = self.add_rc(&current_state, P::ROUND_CONSTANTS[r]);
            current_state = self.sbox(&current_state);
            self.matmul_external(&mut current_state);
        }
        current_state
    }

    // TODO#q: rename to external sbox
    fn sbox(&self, input: &[F]) -> Vec<F> {
        input.iter().map(|el| self.sbox_p(el)).collect()
    }

    // TODO#q: rename to internal sbox
    fn sbox_p(&self, input: &F) -> F {
        input.pow(P::D)
    }

    fn matmul_m4(&self, input: &mut [F]) {
        let t = P::T;
        let t4 = t / 4;
        for i in 0..t4 {
            let start_index = i * 4;
            let mut t_0 = input[start_index];
            t_0.add_assign(&input[start_index + 1]);
            let mut t_1 = input[start_index + 2];
            t_1.add_assign(&input[start_index + 3]);
            let mut t_2 = input[start_index + 1];
            t_2.double_in_place();
            t_2.add_assign(&t_1);
            let mut t_3 = input[start_index + 3];
            t_3.double_in_place();
            t_3.add_assign(&t_0);
            let mut t_4 = t_1;
            t_4.double_in_place();
            t_4.double_in_place();
            t_4.add_assign(&t_3);
            let mut t_5 = t_0;
            t_5.double_in_place();
            t_5.double_in_place();
            t_5.add_assign(&t_2);
            let mut t_6 = t_3;
            t_6.add_assign(&t_5);
            let mut t_7 = t_2;
            t_7.add_assign(&t_4);
            input[start_index] = t_6;
            input[start_index + 1] = t_5;
            input[start_index + 2] = t_7;
            input[start_index + 3] = t_4;
        }
    }

    /// Apply the external MDS matrix M_E to the state
    fn matmul_external(&self, input: &mut [F]) {
        let t = P::T;
        match t {
            2 => {
                // Matrix circ(2, 1)
                let mut sum = input[0];
                sum.add_assign(&input[1]);
                input[0].add_assign(&sum);
                input[1].add_assign(&sum);
            }
            3 => {
                // Matrix circ(2, 1, 1)
                let mut sum = input[0];
                sum.add_assign(&input[1]);
                sum.add_assign(&input[2]);
                input[0].add_assign(&sum);
                input[1].add_assign(&sum);
                input[2].add_assign(&sum);
            }
            4 => {
                // Applying cheap 4x4 MDS matrix to each 4-element part of the
                // state
                self.matmul_m4(input);
            }
            8 | 12 | 16 | 20 | 24 => {
                // Applying cheap 4x4 MDS matrix to each 4-element part of the
                // state
                self.matmul_m4(input);

                // Applying second cheap matrix for t > 4
                let t4 = t / 4;
                let mut stored = [F::zero(); 4];
                for l in 0..4 {
                    stored[l] = input[l];
                    for j in 1..t4 {
                        stored[l].add_assign(&input[4 * j + l]);
                    }
                }
                for i in 0..input.len() {
                    input[i].add_assign(&stored[i % 4]);
                }
            }
            _ => {
                panic!()
            }
        }
    }

    /// Apply the internal MDS matrix M_I to the state
    fn matmul_internal(&self, input: &mut [F], mat_internal_diag_m_1: &[F]) {
        let t = P::T;

        match t {
            2 => {
                // [2, 1]
                // [1, 3]
                let mut sum = input[0];
                sum.add_assign(&input[1]);
                input[0].add_assign(&sum);
                input[1].double_in_place();
                input[1].add_assign(&sum);
            }
            3 => {
                // [2, 1, 1]
                // [1, 2, 1]
                // [1, 1, 3]
                let mut sum = input[0];
                sum.add_assign(&input[1]);
                sum.add_assign(&input[2]);
                input[0].add_assign(&sum);
                input[1].add_assign(&sum);
                input[2].double_in_place();
                input[2].add_assign(&sum);
            }
            4 | 8 | 12 | 16 | 20 | 24 => {
                // Compute input sum
                let mut sum = input[0];
                input
                    .iter()
                    .skip(1)
                    .take(t - 1)
                    .for_each(|el| sum.add_assign(el));
                // Add sum + diag entry * element to each element
                for i in 0..input.len() {
                    input[i].mul_assign(&mat_internal_diag_m_1[i]);
                    input[i].add_assign(&sum);
                }
            }
            _ => {
                panic!()
            }
        }
    }

    /// Add a round constant to the state.
    fn add_rc(&self, input: &[F], rc: &[F]) -> Vec<F> {
        input
            .iter()
            .zip(rc.iter())
            .map(|(a, b)| {
                let mut r = *a;
                r.add_assign(b);
                r
            })
            .collect()
    }
}

pub trait MerkleTreeHash<F: PrimeField> {
    fn compress(&self, input: &[&F]) -> F;
}

impl<P: PoseidonParams<F>, F: PrimeField> MerkleTreeHash<F>
    for Poseidon2<P, F>
{
    fn compress(&self, input: &[&F]) -> F {
        self.permutation(&[input[0].to_owned(), input[1].to_owned(), F::zero()])
            [0]
    }
}

#[allow(unused_imports)]
#[cfg(test)]
pub fn random_scalar<F: PrimeField + crypto_bigint::Random>() -> F {
    let mut rng = rand::thread_rng();
    F::random(&mut rng)
}
