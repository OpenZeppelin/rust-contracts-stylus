//! [EIP-712](https://eips.ethereum.org/EIPS/eip-712) is a standard for hashing
//! and signing typed structured data.
//!
//! The implementation of the domain separator was designed to be as efficient
//! as possible while still properly updating the chain id to protect against
//! replay attacks on an eventual fork of the chain.
//!
//! NOTE: This contract implements the version of the encoding known as "v4", as
//! implemented by the JSON RPC method [`eth_signTypedDataV4`] in MetaMask.
//!
//! [`eth_signTypedDataV4`]: https://docs.metamask.io/guide/signing-data.html

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

/// `keccak256("EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)");`
pub const TYPE_HASH: [u8; 32] =
    hex!("8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f");
pub const FIELDS: [u8; 1] = hex!("0f");
pub const SALT: [u8; 32] = [0u8; 32];

pub type DomainSeparatorTuple = sol! {
    tuple(bytes32, bytes32, bytes32, uint256, address)
};

sol_storage! {
    /// State of an `Eip712` contract.
    pub struct Eip712 {
        /// The cached domain separator.
        bytes32 _cached_domain_separator;
        /// The cached chain id.
        uint256 _cached_chain_id;
        /// The cached contract address, i.e. `address(this)`.
        address _cached_this;
        /// The hashed name.
        bytes32 _hashed_name;
        /// The hashed version.
        bytes32 _hashed_version;
        /// The name.
        string _name;
        /// The version.
        string _version;
    }
}

#[external]
impl Eip712 {
    /// Returns the fields and values that describe the domain separator used by
    /// this contract for EIP-712 signature.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
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

impl Eip712 {
    /// Returns the domain separator for the current chain [not using cache].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
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
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
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

    /// Given an already [hashed struct], this function returns the hash of the
    /// fully encoded EIP-721 message for this domain.
    ///
    /// [hashed struct]: https://eips.ethereum.org/EIPS/eip-712#definition-of-hashstruct
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn hash_typed_data_v4(&self, _hash_struct: B256) -> B256 {
        let _domain_separator = self.domain_separator_v4();
        to_typed_data_hash(_domain_separator, _hash_struct)
    }

    /// The name parameter for the EIP-721 domain.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn eip712_name(&self) -> String {
        self._name.get_string()
    }

    /// The version parameter for the EIP-721 domain.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    pub fn eip712_version(&self) -> String {
        self._version.get_string()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    const NAME: &str = "EIP712";
    const VERSION: &str = "1";

    #[motsu::test]
    fn test_eip712_name(contract: Eip712) {
        contract._name.set_str(NAME);

        assert_eq!(contract.eip712_name(), NAME);
    }

    #[motsu::test]
    fn test_eip712_version(contract: Eip712) {
        contract._version.set_str(VERSION);

        assert_eq!(contract.eip712_version(), VERSION);
    }

    #[motsu::test]
    fn test_eip712_domain(contract: Eip712) {
        contract._name.set_str(NAME);
        contract._version.set_str(VERSION);

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
                NAME.to_string(),
                VERSION.to_string(),
                chain_id,
                verifying_contract,
                SALT,
                Vec::new()
            )
        );
    }

    #[motsu::test]
    fn test_build_domain_separator(contract: Eip712) {
        contract._name.set_str(NAME);
        contract._version.set_str(VERSION);

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
    fn test_domain_separator_v4(contract: Eip712) {
        contract._name.set_str(NAME);
        contract._version.set_str(VERSION);

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
    fn test_hash_typed_data_v4(contract: Eip712) {
        contract._name.set_str(NAME);
        contract._version.set_str(VERSION);

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
