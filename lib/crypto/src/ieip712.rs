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
use hex_literal::hex;

use crate::message_hash_utils::to_typed_data_hash;

/// keccak256("EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)")
pub const TYPE_HASH: [u8; 32] =
    hex!("8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f");
/// Field for the domain separator. `hex"0f"`
pub const FIELDS: [u8; 1] = hex!("0f");
/// Salt for the domain separator. `bytes32(0)`
pub const SALT: [u8; 32] = [0u8; 32];
/// Tuple for the domain separator.
pub type DomainSeparatorTuple = sol! {
    tuple(bytes32, bytes32, bytes32, uint256, address)
};

/// EIP-712 Contract interface.
pub trait IEIP712 {
    /// Immutable name of EIP-712 instance.
    const NAME: &'static str;
    /// Immutable version of EIP-712 instance.
    const VERSION: &'static str;
    /// Returns chain id.
    fn chain_id() -> u64;
    /// Returns the contract's address.
    fn contract_address() -> Address;

    /// Returns the fields and values that describe the domain separator used by
    /// this contract for EIP-712 signature.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn eip712_domain(
        &self,
    ) -> ([u8; 1], String, String, u64, Address, [u8; 32], Vec<U256>) {
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
    /// This function employs a cache to avoid recomputing the domain separator.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn domain_separator_v4(&self) -> B256 {
        let hashed_name = keccak256(Self::NAME.as_bytes());
        let hashed_version = keccak256(Self::VERSION.as_bytes());

        let encoded = DomainSeparatorTuple::encode_params(&(
            TYPE_HASH,
            *hashed_name,
            *hashed_version,
            U256::from(Self::chain_id()),
            Self::contract_address(),
        ));

        keccak256(encoded)
    }

    /// Given an already [hashed struct], this function returns the hash of the
    /// fully encoded EIP-721 message for this domain.
    ///
    /// [hashed struct]: https://eips.ethereum.org/EIPS/eip-712#definition-of-hashstruct
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn hash_typed_data_v4(&self, hash_struct: B256) -> B256 {
        let domain_separator = self.domain_separator_v4();
        to_typed_data_hash(domain_separator, hash_struct)
    }
}
