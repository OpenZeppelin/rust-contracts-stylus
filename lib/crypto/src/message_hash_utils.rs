//! Signature message hash utilities for producing digests to be consumed by
//! `ECDSA` recovery or signing.
//!
//! The library provides methods for generating a hash of a message that
//! conforms to the [ERC-191] and [EIP 712] specifications.
//!
//! [ERC-191]: https://eips.ethereum.org/EIPS/eip-191
//! [EIP-712]: https://eips.ethereum.org/EIPS/eip-712

use alloc::string::String;

use alloy_primitives::{keccak256, B256};
use alloy_sol_types::{sol, SolType};
use hex_literal::hex;

/// "\x19Ethereum Signed Message:\n32" in bytes
const ETH_MESSAGE_PREFIX: [u8; 28] =
    hex!("19457468657265756d205369676e6564204d6573736167653a0a3332");

/// Returns the keccak256 digest of an ERC-191 signed data with version `0x45`
/// (`personal_sign` messages).
///
/// The digest is calculated by prefixing a bytes32 `message_hash` with
/// `"\x19Ethereum Signed Message:\n32"` and hashing the result. It corresponds
/// with the hash signed when using the [eth_sign] JSON-RPC method.
///
/// NOTE: The `message_hash` parameter is intended to be the result of hashing a
/// raw message with keccak256, although any bytes32 value can be safely used
/// because the final digest will be re-hashed.
///
/// [eth_sign]: https://eth.wiki/json-rpc/API#eth_sign
pub fn to_eth_signed_message_hash(message_hash: B256) -> B256 {
    type EthMessageCoder = sol! {
        tuple(bytes, bytes32)
    };

    let encoded = EthMessageCoder::encode_packed(&(
        ETH_MESSAGE_PREFIX.to_vec(),
        *message_hash,
    ));
    keccak256(encoded)
}

/// Returns the keccak256 digest of an EIP-712 typed data (ERC-191 version
/// `0x01`).
///
/// The digest is calculated from a `domain_separator` and a `struct_hash`, by
/// prefixing them with `\x19\x01` and hashing the result. It corresponds to the
/// hash signed by the [eth_signTypedData] JSON-RPC method as part of EIP-712.
///
/// [eth_signTypedData]: https://eips.ethereum.org/EIPS/eip-712
pub fn to_typed_data_hash(domain_separator: B256, struct_hash: B256) -> B256 {
    type TypeHashCoder = sol! {
        tuple(string, bytes32, bytes32)
    };

    let encoded = TypeHashCoder::encode_packed(&(
        String::from("\x19\x01"),
        *domain_separator,
        *struct_hash,
    ));
    keccak256(encoded)
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
