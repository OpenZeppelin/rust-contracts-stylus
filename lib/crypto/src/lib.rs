/*!
Common cryptographic procedures for a blockchain environment.

> Note that `crypto` is still `0.*.*`, so breaking changes
> [may occur at any time](https://semver.org/#spec-item-4). If you must depend
> on `crypto`, we recommend pinning to a specific version, i.e., `=0.y.z`.

## Verifying Merkle Proofs

[`merkle.rs`](./src/merkle.rs) provides:

- A `verify` function which can prove that some value is part of a
  [Merkle tree].
- A `verify_multi_proof` function which can prove multiple values are part of a
  [Merkle tree].

[Merkle tree]: https://en.wikipedia.org/wiki/Merkle_tree

*/

#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;
extern crate core;

#[macro_use]
pub mod biginteger;
pub use biginteger::{
    signed_mod_reduction, BigInt, BigInteger, BigInteger128, BigInteger256,
    BigInteger320, BigInteger384, BigInteger448, BigInteger64, BigInteger768,
    BigInteger832,
};
pub mod bits;
pub mod const_helpers;
#[macro_use]
pub mod field;
pub mod hash;
pub mod keccak;
pub mod merkle;
pub mod poseidon2;

pub use keccak::KeccakBuilder;
