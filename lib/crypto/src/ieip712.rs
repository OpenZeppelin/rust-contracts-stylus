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

use alloy_primitives::{hex, Address, B256, U256};
use alloy_sol_types::{sol, SolType};

use crate::{
    hash::{BuildHasher, Hasher},
    message_hash_utils::to_typed_data_hash,
    KeccakBuilder,
};

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
    fn chain_id() -> U256;
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
    /// This function employs a cache to avoid recomputing the domain separator.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn domain_separator_v4(&self) -> B256 {
        let b = KeccakBuilder;
        let mut name_hasher = b.build_hasher();
        name_hasher.update(Self::NAME.as_bytes());
        let hashed_name = name_hasher.finalize();

        let mut version_hasher = b.build_hasher();
        version_hasher.update(Self::VERSION.as_bytes());
        let hashed_version = version_hasher.finalize();

        let encoded = DomainSeparatorTuple::encode_params(&(
            TYPE_HASH,
            hashed_name,
            hashed_version,
            Self::chain_id(),
            Self::contract_address(),
        ));

        let mut domain_separator_hasher = b.build_hasher();
        domain_separator_hasher.update(encoded);
        domain_separator_hasher.finalize().into()
    }

    /// Given an already [hashed struct], this function returns the hash of the
    /// fully encoded EIP-712 message for this domain.
    ///
    /// [hashed struct]: https://eips.ethereum.org/EIPS/eip-712#definition-of-hashstruct
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn hash_typed_data_v4(&self, hash_struct: B256) -> B256 {
        let domain_separator = self.domain_separator_v4();
        to_typed_data_hash(&domain_separator, &hash_struct).into()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};

    use super::{FIELDS, IEIP712, SALT};

    const CHAIN_ID: U256 = uint!(42161_U256);
    const CONTRACT_ADDRESS: Address =
        address!("dCE82b5f92C98F27F116F70491a487EFFDb6a2a9");

    #[derive(Default)]
    struct TestEIP712 {}

    impl IEIP712 for TestEIP712 {
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
        let contract = TestEIP712::default();
        let domain = contract.eip712_domain();
        assert_eq!(FIELDS, domain.0);
        assert_eq!(TestEIP712::NAME, domain.1);
        assert_eq!(TestEIP712::VERSION, domain.2);
        assert_eq!(CHAIN_ID, domain.3);
        assert_eq!(CONTRACT_ADDRESS, domain.4);
        assert_eq!(SALT, domain.5);
        assert_eq!(Vec::<U256>::new(), domain.6);
    }
}
