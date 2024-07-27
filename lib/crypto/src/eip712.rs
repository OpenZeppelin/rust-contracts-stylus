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

use alloc::borrow::ToOwned;
use alloc::{string::String, vec::Vec};
use alloy_primitives::{
    b256, fixed_bytes, keccak256, Address, FixedBytes, B256, U256,
};
use hex_literal::hex;

/// keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")
pub const TYPE_HASH: [u8; 32] =
    hex!("8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f");
/// Field for the domain separator. `hex"0f"`
pub const FIELDS: [u8; 1] = hex!("0f");
/// Salt for the domain separator. `bytes32(0)`
pub const SALT: [u8; 32] = [0u8; 32];

/// EIP712 contract trait.
pub trait EIP712 {
    /// Immutable name of EIP-712 instance.
    const NAME: &'static str;
    /// Immutable version of EIP-712 instance.
    const VERSION: &'static str;
    /// This is Abritrum's chain id. Provided by the consuming contract.
    const CHAIN_ID: u64;
    /// This is the contract address of the consuming contract.
    const CONTRACT_ADDRESS: Address;

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
            Self::CHAIN_ID,
            Self::CONTRACT_ADDRESS,
            SALT,
            Vec::new(),
        )
    }
}
