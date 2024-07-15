//! [EIP-712](https://eips.ethereum.org/EIPS/eip-712) is a standard for hashing and signing of typed structured data.
//!
//! The encoding scheme specified in the EIP requires a domain separator and a hash of the typed structured data,
//! whose encoding is very generic and therefore its implementation in Solidity is not feasible.
//! Thus, this contract does not implement the encoding itself. Protocols need to implement the type-specific encoding
//! they need in order to produce the hash of their typed data using a combination of `abi.encode` and `keccak256`.
//!
//! This contract implements the EIP-712 domain separator (`_domainSeparatorV4`) that is used as part of the encoding
//! scheme, and the final step of the encoding to obtain the message digest that is then signed via ECDSA
//! (`_hashTypedDataV4`).
//!
//! The implementation of the domain separator was designed to be as efficient as possible while still properly updating
//! the chain id to protect against replay attacks on an eventual fork of the chain.
//!
//! NOTE: This contract implements the version of the encoding known as "v4", as implemented by the JSON RPC method
//! [`eth_signTypedDataV4` in MetaMask](https://docs.metamask.io/guide/signing-data.html).
//!
//! NOTE: In the upgradeable version of this contract, the cached values will correspond to the address and the domain
//! separator of the implementation contract. This will cause the `_domainSeparatorV4` function to always rebuild the
//! separator from the immutable values, which is cheaper than accessing a cached version in cold storage.
//!
//! # Custom Attributes
//! 
//! * `oz-upgrades-unsafe-allow state-variable-immutable`

const EIP712: &str = "";