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

// TODO: Find a way for removing `alloy_primitives` crate from lib/crypto.
// Here we need a workaround for `U256`.
use alloy_primitives::U256;
use alloy_sol_types::{sol, SolType};

use crate::{
    hash::{BuildHasher, Hasher},
    message_hash_utils::to_typed_data_hash,
    Address, Bytes1, Bytes32, KeccakBuilder,
};

/// keccak256("EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)")
pub const TYPE_HASH: Bytes32 = [
    0x8b, 0x73, 0xc3, 0xc6, 0x9b, 0xb8, 0xfe, 0x3d, 0x51, 0x2e, 0xcc, 0x4c,
    0xf7, 0x59, 0xcc, 0x79, 0x23, 0x9f, 0x7b, 0x17, 0x9b, 0x0f, 0xfa, 0xca,
    0xa9, 0xa7, 0x5d, 0x52, 0x2b, 0x39, 0x40, 0x0f,
];

/// Field for the domain separator.
pub const FIELDS: Bytes1 = [0x0f];

/// Salt for the domain separator.
pub const SALT: Bytes32 = [0u8; 32];

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
    ) -> (Bytes1, String, String, U256, Address, Bytes32, Vec<U256>) {
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
    fn domain_separator_v4(&self) -> Bytes32 {
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
            Self::contract_address().into(),
        ));

        let mut domain_separator_hasher = b.build_hasher();
        domain_separator_hasher.update(encoded);
        domain_separator_hasher.finalize()
    }

    /// Given an already [hashed struct], this function returns the hash of the
    /// fully encoded EIP-712 message for this domain.
    ///
    /// [hashed struct]: https://eips.ethereum.org/EIPS/eip-712#definition-of-hashstruct
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn hash_typed_data_v4(&self, hash_struct: Bytes32) -> Bytes32 {
        let domain_separator = self.domain_separator_v4();
        to_typed_data_hash(&domain_separator, &hash_struct)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{uint, U256};

    use super::{FIELDS, IEIP712, SALT};
    use crate::Address;

    const CHAIN_ID: U256 = uint!(42161_U256);

    const CONTRACT_ADDRESS: Address = [
        0xdC, 0xE8, 0x2b, 0x5f, 0x92, 0xC9, 0x8F, 0x27, 0xF1, 0x16, 0xF7, 0x04,
        0x91, 0xa4, 0x87, 0xEF, 0xFD, 0xb6, 0xa2, 0xa9,
    ];

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
