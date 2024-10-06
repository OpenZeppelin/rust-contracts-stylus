// TODO#q: make sense to use ark_ff at first

// use ark_ff::fields::{Fp256, MontBackend, MontConfig};
//
// #[derive(MontConfig)]
// #[modulus =
// "21888242871839275222246405745257275088548364400416034343698204186575808495617"
// ] #[generator = "5"]
// pub struct FrConfig;
// pub type Fr = Fp256<MontBackend<FrConfig, 4>>;

// TODO#q: Use https://crates.io/crates/num-bigint crate for bigint numbers

pub mod merkle_tree_fp;
pub mod utils;
pub mod vesta;
