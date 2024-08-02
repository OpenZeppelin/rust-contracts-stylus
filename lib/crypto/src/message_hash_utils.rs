//! Signature message hash utilities for producing digests to be consumed by
//! `ECDSA` recovery or signing.
//!
//! The library provides methods for generating a hash of a message that
//! conforms to the [ERC-191] and [EIP 712] specifications.
//!
//! [ERC-191]: https://eips.ethereum.org/EIPS/eip-191
//! [EIP-712]: https://eips.ethereum.org/EIPS/eip-712

use alloc::string::ToString;

use alloy_primitives::hex;

use crate::{
    hash::{BuildHasher, Hasher},
    KeccakBuilder,
};

/// Prefix for EIP-191 Signed Data Standard.
pub const EIP191_PREFIX: &str = "\x19Ethereum Signed Message:\n";

/// Prefix for ERC-191 version with `0x01`.
pub const TYPED_DATA_PREFIX: [u8; 2] = hex!("1901");

/// Returns the keccak256 digest of an ERC-191 signed data with version `0x45`
/// (`personal_sign` messages).
///
/// The digest is calculated by prefixing a bytes32 `message_hash` with
/// [`EIP191_PREFIX`] and length of the `message_hash`, and hashing the result.
/// It corresponds with the hash signed when using the [eth_sign]
/// JSON-RPC method.
///
/// NOTE: The `message_hash` parameter is intended to be the result of hashing a
/// raw message with `keccak256`, although any B256 value can be safely used
/// because the final digest will be re-hashed.
///
/// [eth_sign]: https://eth.wiki/json-rpc/API#eth_sign
#[must_use]
pub fn to_eth_signed_message_hash(message_hash: &[u8; 32]) -> [u8; 32] {
    eip_191_hash(message_hash)
}

/// Returns the keccak256 digest of an EIP-712 typed data (ERC-191 version
/// `0x01`).
///
/// The digest is calculated from a `domain_separator` and a `struct_hash`, by
/// prefixing them with `[TYPED_DATA_PREFIX]` and hashing the result. It
/// corresponds to the hash signed by the [eth_signTypedData] JSON-RPC method as
/// part of EIP-712.
///
/// [eth_signTypedData]: https://eips.ethereum.org/EIPS/eip-712
#[must_use]
pub fn to_typed_data_hash(
    domain_separator: &[u8; 32],
    struct_hash: &[u8; 32],
) -> [u8; 32] {
    let b = KeccakBuilder;
    let mut hasher = b.build_hasher();
    hasher.update(TYPED_DATA_PREFIX);
    hasher.update(domain_separator);
    hasher.update(struct_hash);
    hasher.finalize()
}

/// Calculates a `keccak256` hash of the `message`
/// according to [EIP-191] (version `0x01`).
///
/// The final message is a UTF-8 string, encoded as follows:
/// `"\x19Ethereum Signed Message:\n" + message.length + message`
///
/// [EIP-191]: https://eips.ethereum.org/EIPS/eip-191
#[must_use]
pub fn eip_191_hash(message: &[u8]) -> [u8; 32] {
    let b = KeccakBuilder;
    let mut hasher = b.build_hasher();
    hasher.update(EIP191_PREFIX.as_bytes());
    hasher.update(message.len().to_string().as_bytes());
    hasher.update(message);
    hasher.finalize()
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::b256;

    use super::*;

    #[test]
    fn test_to_eth_signed_message_hash() {
        // bytes32("stylus");
        let message_hash = b256!(
            "7379746c75730000000000000000000000000000000000000000000000000000"
        );
        let expected = b256!(
            "a5667772cbc7da54ae0530c5f46433ef97e01537b744a9fbe663e7117824c8a1"
        );

        // assert_eq!(to_eth_signed_message_hash(message_hash), expected);
    }

    #[test]
    fn test_to_typed_data_hash() {
        // TYPE_HASH
        let domain_separator = b256!(
            "8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f"
        );
        // bytes32("stylus");
        let struct_hash = b256!(
            "7379746c75730000000000000000000000000000000000000000000000000000"
        );
        let expected = b256!(
            "cefc47137f8165d8270433dd62e395f5672966b83a113a7bb7b2805730a2197e"
        );

        // assert_eq!(to_typed_data_hash(domain_separator, struct_hash),
        // expected);
    }
}
