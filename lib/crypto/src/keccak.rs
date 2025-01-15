//! An interface to the default hashing algorithm used in this library's [merkle
//! proofs][crate].
use tiny_keccak::{Hasher as TinyHasher, Keccak};

use crate::hash::{BuildHasher, Hash, Hasher};

/// The default [`Hasher`] builder used in this library's [merkle
/// proofs][crate].
///
/// It instantiates a [`Keccak256`] hasher.
pub struct KeccakBuilder;

impl BuildHasher for KeccakBuilder {
    type Hasher = Keccak256;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Keccak256(Keccak::v256())
    }
}

/// The default [`Hasher`] used in this library's [merkle proofs][crate].
///
/// The underlying implementation is guaranteed to match that of the
/// `keccak256` algorithm, commonly used in Ethereum.
pub struct Keccak256(Keccak);

impl Hasher for Keccak256 {
    type Output = [u8; 32];

    fn update(&mut self, input: impl AsRef<[u8]>) {
        self.0.update(input.as_ref());
    }

    fn finalize(self) -> Self::Output {
        let mut buffer = [0u8; 32];
        self.0.finalize(&mut buffer);
        buffer
    }
}

impl Hash for [u8; 32] {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.update(self);
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn sequential_updates_match_concatenated(data1: Vec<u8>, data2: Vec<u8>) {
            let builder = KeccakBuilder;

            let mut hasher1 = builder.build_hasher();
            hasher1.update(&data1);
            hasher1.update(&data2);
            let result1 = hasher1.finalize();

            let mut hasher2 = builder.build_hasher();
            let mut concatenated = data1.clone();
            concatenated.extend_from_slice(&data2);
            hasher2.update(concatenated);
            let result2 = hasher2.finalize();

            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn split_updates_match_full_update(data: Vec<u8>, split_point: usize) {
            if data.is_empty() {
                return Ok(());
            }

            let builder = KeccakBuilder;
            let split_at = split_point % data.len();

            let mut hasher1 = builder.build_hasher();
            hasher1.update(&data[..split_at]);
            hasher1.update(&data[split_at..]);
            let result1 = hasher1.finalize();

            let mut hasher2 = builder.build_hasher();
            hasher2.update(&data);
            let result2 = hasher2.finalize();

            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn multiple_hasher_instances_are_consistent(
            data1: Vec<u8>,
            data2: Vec<u8>,
        ) {
            let builder = KeccakBuilder;

            let mut hasher1 = builder.build_hasher();
            hasher1.update(&data1);
            hasher1.update(&data2);
            let result1 = hasher1.finalize();

            let mut hasher2 = builder.build_hasher();
            hasher2.update(&data1);
            hasher2.update(&data2);
            let result2 = hasher2.finalize();

            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn output_is_always_32_bytes(data: Vec<u8>) {
            let builder = KeccakBuilder;
            let mut hasher = builder.build_hasher();
            hasher.update(&data);
            let result = hasher.finalize();
            assert_eq!(result.len(), 32);
        }
    }

    #[test]
    fn test_empty_input() {
        let builder = KeccakBuilder;
        let mut hasher = builder.build_hasher();
        hasher.update(&[]);
        let result = hasher.finalize();
        let expected: [u8; 32] = [
            0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d,
            0xb2, 0xdc, 0xc7, 0x03, 0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82,
            0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_known_hash() {
        let builder = KeccakBuilder;
        let mut hasher = builder.build_hasher();
        hasher.update(b"hello");
        let result = hasher.finalize();
        let expected: [u8; 32] = [
            0x1c, 0x8a, 0xff, 0x95, 0x06, 0x85, 0xc2, 0xed, 0x4b, 0xc3, 0x17,
            0x4f, 0x34, 0x72, 0x28, 0x7b, 0x56, 0xd9, 0x51, 0x7b, 0x9c, 0x94,
            0x81, 0x27, 0x31, 0x9a, 0x09, 0xa7, 0xa3, 0x6d, 0xea, 0xc8,
        ];
        assert_eq!(result, expected);
    }
}
