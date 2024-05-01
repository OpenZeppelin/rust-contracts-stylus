#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

#[cfg(any(feature = "std", feature = "merkle"))]
pub mod merkle;
