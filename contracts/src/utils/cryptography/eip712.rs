//! [EIP-712](https://eips.ethereum.org/EIPS/eip-712) is a standard for hashing
//! and signing typed structured data.
//!
//! The implementation of the domain separator was designed to be as efficient
//! as possible while still properly updating the chain id to protect against
//! replay attacks on an eventual fork of the chain.
//!
//! NOTE: This contract implements the version of the encoding known as "v4", as
//! implemented by the JSON RPC method [`eth_signTypedDataV4`] in `MetaMask`.
//!
//! [`eth_signTypedDataV4`]: https://docs.metamask.io/guide/signing-data.html

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use alloy_primitives::{keccak256, Address, B256, U256};
use alloy_sol_types::{sol, SolType};
use stylus_sdk::{block, contract};

/// Keccak-256 hash of the EIP-712 domain separator type string.
const TYPE_HASH: [u8; 32] =
    keccak_const::Keccak256::new()
        .update(b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
        .finalize();

/// Field for the domain separator.
const FIELDS: [u8; 1] = [0x0f];

/// Salt for the domain separator.
const SALT: [u8; 32] = [0u8; 32];

/// Prefix for ERC-191 version with `0x01`.
const TYPED_DATA_PREFIX: [u8; 2] = [0x19, 0x01];

/// Tuple for the domain separator.
type DomainSeparatorTuple = sol! {
    tuple(bytes32, bytes32, bytes32, uint256, address)
};

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
) -> B256 {
    let mut preimage = [0u8; 66];
    preimage[..2].copy_from_slice(&TYPED_DATA_PREFIX);
    preimage[2..34].copy_from_slice(domain_separator);
    preimage[34..].copy_from_slice(struct_hash);
    keccak256(preimage)
}

/// EIP-712 Contract interface.
pub trait IEip712 {
    /// Immutable name of EIP-712 instance.
    const NAME: &'static str;
    /// Hashed name of EIP-712 instance.
    const HASHED_NAME: [u8; 32] =
        keccak_const::Keccak256::new().update(Self::NAME.as_bytes()).finalize();

    /// Immutable version of EIP-712 instance.
    const VERSION: &'static str;
    /// Hashed version of EIP-712 instance.
    const HASHED_VERSION: [u8; 32] = keccak_const::Keccak256::new()
        .update(Self::VERSION.as_bytes())
        .finalize();

    /// Returns chain id.
    #[must_use]
    fn chain_id() -> U256 {
        U256::from(block::chainid())
    }

    /// Returns the contract's address.
    #[must_use]
    fn contract_address() -> Address {
        contract::address()
    }

    /// Returns the fields and values that describe the domain separator used by
    /// this contract for EIP-712 signature.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn eip712_domain(
        &self,
    ) -> ([u8; 1], String, String, U256, Address, [u8; 32], Vec<U256>) {
        (
            FIELDS,
            Self::NAME.to_owned(),
            Self::VERSION.to_owned(),
            Self::chain_id(),
            Self::contract_address(),
            SALT,
            Vec::new(),
        )
    }

    /// Returns the domain separator for the current chain.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn domain_separator_v4(&self) -> B256 {
        let encoded = DomainSeparatorTuple::abi_encode(&(
            TYPE_HASH,
            Self::HASHED_NAME,
            Self::HASHED_VERSION,
            Self::chain_id(),
            Self::contract_address(),
        ));

        keccak256(encoded)
    }

    /// Given an already [hashed struct], this function returns the hash of the
    /// fully encoded EIP-712 message for this domain.
    ///
    /// [hashed struct]: https://eips.ethereum.org/EIPS/eip-712#definition-of-hashstruct
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn hash_typed_data_v4(&self, struct_hash: B256) -> B256 {
        let domain_separator = self.domain_separator_v4();
        to_typed_data_hash(&domain_separator, &struct_hash)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, b256, uint, Address, U256};

    use super::{to_typed_data_hash, IEip712, FIELDS, SALT};

    const CHAIN_ID: U256 = uint!(42161_U256);

    const CONTRACT_ADDRESS: Address =
        address!("000000000000000000000000000000000000dEaD");

    #[derive(Default)]
    struct TestEIP712;

    impl IEip712 for TestEIP712 {
        const NAME: &'static str = "A Name";
        const VERSION: &'static str = "1";

        fn chain_id() -> U256 {
            CHAIN_ID
        }

        fn contract_address() -> Address {
            CONTRACT_ADDRESS
        }
    }

    #[test]
    fn domain_test() {
        let contract = TestEIP712;
        let domain = contract.eip712_domain();
        assert_eq!(FIELDS, domain.0);
        assert_eq!(TestEIP712::NAME, domain.1);
        assert_eq!(TestEIP712::VERSION, domain.2);
        assert_eq!(CHAIN_ID, domain.3);
        assert_eq!(CONTRACT_ADDRESS, domain.4);
        assert_eq!(SALT, domain.5);
        assert_eq!(Vec::<U256>::new(), domain.6);
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

        assert_eq!(
            expected,
            to_typed_data_hash(&domain_separator, &struct_hash),
        );
    }
}
