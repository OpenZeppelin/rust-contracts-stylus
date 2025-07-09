//! Module with a contract that implement an upgradeable proxy.
//!
//! It is upgradeable because calls are delegated to an implementation address
//! that can be changed. This address is stored in storage in the location
//! specified by [ERC-1967], so that it doesn't conflict with the storage layout
//! of the implementation behind the proxy.
//!
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967
use alloc::{vec, vec::Vec};

use alloy_primitives::Address;
use stylus_sdk::{
    abi::Bytes,
    prelude::{public, storage},
    ArbResult,
};

use crate::proxy::{erc1967::utils::Erc1967Utils, IProxy};

/// TODO
pub struct Erc1967Proxy {}

impl Erc1967Proxy {}
