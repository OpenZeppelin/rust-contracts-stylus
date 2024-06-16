#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

pub mod hash;
pub use hash::keccak::KeccakBuilder;

#[cfg(any(feature = "std", feature = "ec"))]
#[path = "elliptic-curve/mod.rs"]
pub mod elliptic_curve;

#[cfg(any(feature = "std", feature = "ecdsa"))]
pub mod ecdsa;
#[cfg(any(feature = "std", feature = "ecdsa"))]
pub mod signature;

#[cfg(any(feature = "std", feature = "merkle"))]
pub mod merkle;
