// NOTE#q: this utils ported from root of poseidon2 crate

// use core::cmp::min;
use alloc::{borrow::ToOwned, vec, vec::Vec};

use crate::field::prime::PrimeField;

// TODO#q: use generic factory
// pub fn from_hex<F: PrimeField>(s: &str) -> F {
//     let a = Vec::from_hex!(&s[2..]).expect("Invalid Hex String");
//     F::from_be_bytes_mod_order(&a as &[u8])
// }

// TODO#q: add random
// pub fn random_scalar<F: PrimeField>() -> F {
//     let mut rng = ark_std::rand::thread_rng();
//     F::rand(&mut rng)
// }

// pub fn random_scalar_without_0<F: PrimeField>() -> F {
//     loop {
//         let element = random_scalar::<F>();
//         if !element.is_zero() {
//             return element;
//         }
//     }
// }

//------------------------------------------------------------

// guassian elimination
pub fn mat_inverse<F: PrimeField>(mat: &[Vec<F>]) -> Vec<Vec<F>> {
    let n = mat.len();
    assert!(mat[0].len() == n);

    let mut m = mat.to_owned();
    let mut inv = vec![vec![F::zero(); n]; n];
    for (i, invi) in inv.iter_mut().enumerate() {
        invi[i] = F::one();
    }

    // upper triangle
    for row in 0..n {
        for j in 0..row {
            // subtract from these rows
            let el = m[row][j];
            for col in 0..n {
                // do subtraction for each col
                if col < j {
                    m[row][col] = F::zero();
                } else {
                    let mut tmp = m[j][col];
                    tmp.mul_assign(&el);
                    m[row][col].sub_assign(&tmp);
                }
                if col > row {
                    inv[row][col] = F::zero();
                } else {
                    let mut tmp = inv[j][col];
                    tmp.mul_assign(&el);
                    inv[row][col].sub_assign(&tmp);
                }
            }
        }
        // make 1 in diag
        let el_inv = m[row][row].inverse().unwrap();
        for col in 0..n {
            match col.cmp(&row) {
                core::cmp::Ordering::Less => inv[row][col].mul_assign(&el_inv),
                core::cmp::Ordering::Equal => {
                    m[row][col] = F::one();
                    inv[row][col].mul_assign(&el_inv)
                }
                core::cmp::Ordering::Greater => m[row][col].mul_assign(&el_inv),
            }
        }
    }

    // upper triangle
    for row in (0..n).rev() {
        for j in (row + 1..n).rev() {
            // subtract from these rows
            let el = m[row][j];
            for col in 0..n {
                // do subtraction for each col

                #[cfg(debug_assertions)]
                {
                    if col >= j {
                        m[row][col] = F::zero();
                    }
                }
                let mut tmp = inv[j][col];
                tmp.mul_assign(&el);
                inv[row][col].sub_assign(&tmp);
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        for (row, mrow) in m.iter().enumerate() {
            for (col, v) in mrow.iter().enumerate() {
                if row == col {
                    debug_assert!(*v == F::one());
                } else {
                    debug_assert!(*v == F::zero());
                }
            }
        }
    }

    inv
}

pub fn mat_transpose<F: PrimeField>(mat: &[Vec<F>]) -> Vec<Vec<F>> {
    let rows = mat.len();
    let cols = mat[0].len();
    let mut transpose = vec![vec![F::zero(); rows]; cols];

    for (row, matrow) in mat.iter().enumerate() {
        for col in 0..cols {
            transpose[col][row] = matrow[col];
        }
    }
    transpose
}
