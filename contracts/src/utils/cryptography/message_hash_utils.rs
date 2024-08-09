//! Signature message hash utilities for producing digests to be consumed by
//! `ECDSA` recovery or signing.
//!
//! The library provides methods for generating a hash of a message that
//! conforms to the [EIP 712] specification.
//!
//! [EIP-712]: https://eips.ethereum.org/EIPS/eip-712

use alloy_primitives::{keccak256, FixedBytes};

/// Prefix for ERC-191 version with `0x01`.
pub const TYPED_DATA_PREFIX: [u8; 2] = [0x19, 0x01];

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
) -> FixedBytes<32> {
    let mut preimage = [0u8; 66];
    preimage[..2].copy_from_slice(&TYPED_DATA_PREFIX);
    preimage[2..34].copy_from_slice(domain_separator);
    preimage[34..].copy_from_slice(struct_hash);
    keccak256(preimage)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::b256;

    use super::to_typed_data_hash;

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

        assert_eq!(
            expected,
            to_typed_data_hash(&domain_separator, &struct_hash),
        );
    }
}
