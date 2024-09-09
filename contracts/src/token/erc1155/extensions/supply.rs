//! Extension of ERC-1155 that adds tracking of total supply per id.
//!
//! Useful for scenarios where Fungible and Non-fungible tokens have to be
//! clearly identified. Note: While a totalSupply of 1 might mean the
//! corresponding is an NFT, there is no guarantees that no other token
//! with the same id are not going to be minted.
//!
//! NOTE: This contract implies a global limit of 2**256 - 1 to the number
//! of tokens that can be minted.
//!
//! CAUTION: This extension should not be added in an upgrade to an already
//! deployed contract.
