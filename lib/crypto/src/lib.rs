#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;

#[cfg(feature = "merkle")]
pub mod hash;
#[cfg(feature = "merkle")]
pub mod keccak;
#[cfg(feature = "merkle")]
pub mod merkle;
