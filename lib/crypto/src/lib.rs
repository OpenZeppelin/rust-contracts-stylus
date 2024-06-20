#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

pub mod hash;
#[cfg(any(feature = "std", feature = "merkle"))]
pub mod merkle;

pub mod keccak;
pub use keccak::KeccakBuilder;
