//! [EIP-712](https://eips.ethereum.org/EIPS/eip-712) is a standard for hashing and signing of typed structured data.
//!
//! The encoding scheme specified in the EIP requires a domain separator and a
//! hash of the typed structured data, whose encoding is very generic and
//! therefore its implementation in Solidity is not feasible. Thus, this
//! contract does not implement the encoding itself. Protocols need to implement
//! the type-specific encoding they need in order to produce the hash of their
//! typed data using a combination of `abi.encode` and `keccak256`.
//!
//! This contract implements the EIP-712 domain separator (`_domainSeparatorV4`)
//! that is used as part of the encoding scheme, and the final step of the
//! encoding to obtain the message digest that is then signed via ECDSA
//! (`_hashTypedDataV4`).
//!
//! The implementation of the domain separator was designed to be as efficient
//! as possible while still properly updating the chain id to protect against
//! replay attacks on an eventual fork of the chain.
//!
//! NOTE: This contract implements the version of the encoding known as "v4", as
//! implemented by the JSON RPC method [`eth_signTypedDataV4` in MetaMask](https://docs.metamask.io/guide/signing-data.html).
//!
//! NOTE: In the upgradeable version of this contract, the cached values will
//! correspond to the address and the domain separator of the implementation
//! contract. This will cause the `_domainSeparatorV4` function to always
//! rebuild the separator from the immutable values, which is cheaper than
//! accessing a cached version in cold storage.
//!
//! # Custom Attributes
//!
//! * `oz-upgrades-unsafe-allow state-variable-immutable`

use alloc::{string::String, vec::Vec};

use alloy_primitives::{
    b256, fixed_bytes, keccak256, Address, FixedBytes, B256, U256,
};
use alloy_sol_types::{sol, SolType};
use stylus_sdk::{
    block, contract,
    stylus_proc::{external, sol_storage},
};

use super::message_hash_utils::to_typed_data_hash;

/// keccak256("EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)");
pub const TYPE_HASH: B256 =
    b256!("8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f");
pub const FIELDS: FixedBytes<1> = fixed_bytes!("15");
pub const SALT: B256 = B256::ZERO;

pub type DomainSeparatorTuple = sol! {
    tuple(bytes32, bytes32, bytes32, uint256, address)
};

sol_storage! {
    /// State of an `EIP712` contract.
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct EIP712 {
        /// The cached domain separator.
        bytes32 _cached_domain_separator;
        /// The cached chain id.
        uint256 _cached_chain_id;
        /// The cached contract address. [address(this)]
        address _cached_this;
        /// The hashed name
        bytes32 _hashed_name;
        /// The hashed version
        bytes32 _hashed_version;
        /// The name
        string _name;
        /// The version
        string _version;
    }
}

#[external]
impl EIP712 {
    /// Returns the fields and values that describe the domain separator used by
    /// this contract for EIP-712 signature.
    pub fn eip712_domain(
        &self,
    ) -> (FixedBytes<1>, String, String, u64, Address, B256, Vec<U256>) {
        (
            FIELDS,
            self.eip712_name(),
            self.eip712_version(),
            block::chainid(),
            contract::address(),
            SALT,
            Vec::new(),
        )
    }
}

impl EIP712 {
    /// Returns the domain separator for the current chain [not using cache].
    pub fn build_domain_separator(&self) -> B256 {
        let encoded = DomainSeparatorTuple::encode_params(&(
            *TYPE_HASH,
            **self._hashed_name,
            **self._hashed_version,
            U256::from(block::chainid()),
            contract::address(),
        ));
        keccak256(encoded)
    }

    /// Returns the domain separator for the current chain.
    /// This function employs a cache to avoid recomputing the domain separator.
    pub fn domain_separator_v4(&self) -> B256 {
        let this = contract::address();
        let cached_this = self._cached_this.get();
        let chain_id = U256::from(block::chainid());
        let cached_chain_id = self._cached_chain_id.get();

        if this == cached_this && chain_id == cached_chain_id {
            self._cached_domain_separator.get()
        } else {
            self.build_domain_separator()
        }
    }

    /// Given an already [hashed struct](https://eips.ethereum.org/EIPS/eip-712#definition-of-hashstruct),
    /// this function returns the hash of the fully encoded EIP712 message for
    /// this domain.
    pub fn hash_typed_data_v4(&self, _hash_struct: B256) -> B256 {
        let _domain_separator = self.domain_separator_v4();
        to_typed_data_hash(_domain_separator, _hash_struct)
    }

    /// The name parameter for the EIP712 domain
    pub fn eip712_name(&self) -> String {
        self._name.get_string()
    }

    /// The version parameter for the EIP712 domain
    pub fn eip712_version(&self) -> String {
        self._version.get_string()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[motsu::test]
    fn test_eip712_name(contract: EIP712) {
        let name = "EIP712";
        contract._name.set_str(name);

        assert_eq!(contract.eip712_name(), name);
    }

    #[motsu::test]
    fn test_eip712_version(contract: EIP712) {
        let version = "1";
        contract._version.set_str(version);

        assert_eq!(contract.eip712_version(), version);
    }

    #[motsu::test]
    fn test_eip712_domain(contract: EIP712) {
        let name = "EIP712";
        let version = "1";
        contract._name.set_str(name);
        contract._version.set_str(version);

        let chain_id = block::chainid();
        let verifying_contract = contract::address();
        let expected_domain_encoded = DomainSeparatorTuple::encode_params(&(
            *TYPE_HASH,
            **contract._hashed_name,
            **contract._hashed_version,
            U256::from(chain_id),
            verifying_contract,
        ));
        let domain_separator_expected_cached =
            keccak256(expected_domain_encoded);

        assert_eq!(
            contract.eip712_domain(),
            (
                FIELDS,
                name.to_string(),
                version.to_string(),
                chain_id,
                verifying_contract,
                SALT,
                Vec::new()
            )
        );
    }

    #[motsu::test]
    fn test_build_domain_separator(contract: EIP712) {
        let name = "EIP712";
        let version = "1";
        contract._name.set_str(name);
        contract._version.set_str(version);

        let chain_id = block::chainid();
        let verifying_contract = contract::address();
        let expected_domain_encoded = DomainSeparatorTuple::encode_params(&(
            *TYPE_HASH,
            **contract._hashed_name,
            **contract._hashed_version,
            U256::from(chain_id),
            verifying_contract,
        ));
        let domain_separator_expected = keccak256(expected_domain_encoded);

        assert_eq!(
            contract.build_domain_separator(),
            domain_separator_expected
        );
    }

    #[motsu::test]
    fn test_domain_separator_v4(contract: EIP712) {
        let name = "EIP712";
        let version = "1";
        contract._name.set_str(name);
        contract._version.set_str(version);

        let chain_id = block::chainid();
        let verifying_contract = contract::address();
        let expected_domain_encoded = DomainSeparatorTuple::encode_params(&(
            *TYPE_HASH,
            **contract._hashed_name,
            **contract._hashed_version,
            U256::from(chain_id),
            verifying_contract,
        ));
        let domain_separator_expected = keccak256(expected_domain_encoded);

        assert_eq!(contract.domain_separator_v4(), domain_separator_expected);
    }

    #[motsu::test]
    fn test_hash_typed_data_v4(contract: EIP712) {
        let name = "EIP712";
        let version = "1";
        contract._name.set_str(name);
        contract._version.set_str(version);

        let chain_id = block::chainid();
        let verifying_contract = contract::address();
        let expected_domain_encoded = DomainSeparatorTuple::encode_params(&(
            *TYPE_HASH,
            **contract._hashed_name,
            **contract._hashed_version,
            U256::from(chain_id),
            verifying_contract,
        ));
        let domain_separator_expected = keccak256(expected_domain_encoded);

        let hash_struct = keccak256("test".as_bytes());
        let expected_hash =
            to_typed_data_hash(domain_separator_expected, hash_struct);

        assert_eq!(contract.hash_typed_data_v4(hash_struct), expected_hash);
    }
}
