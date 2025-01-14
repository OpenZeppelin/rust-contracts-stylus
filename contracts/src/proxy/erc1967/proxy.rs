//! Module with a contract that implement an upgradeable proxy.
//!
//! It is upgradeable because calls are delegated to an implementation address
//! that can be changed. This address is stored in storage in the location
//! specified by [ERC-1967], so that it doesn't conflict with the storage layout
//! of the implementation behind the proxy.
//!
//! [ERC-1967]: https://eips.ethereum.org/EIPS/eip-1967
use alloc::vec::Vec;

use alloy_primitives::Address;
use stylus_sdk::{
    abi::Bytes,
    prelude::{public, storage},
    ArbResult,
};

use crate::proxy::{erc1967::utils::IErc1967Utils, IProxy};

#[storage]
/// TODO
pub struct Erc1967Proxy {}

#[public]
impl Erc1967Proxy {
    #[fallback]
    fn fallback(&mut self) -> ArbResult {
        self.do_fallback()
    }
}

impl IProxy for Erc1967Proxy {
    fn delegate(&mut self, _implementation: Address) -> ArbResult {
        todo!()
    }

    fn implementation(&self) -> Address {
        self.get_implementation()
    }

    fn do_fallback(&mut self) -> ArbResult {
        self.delegate(self.implementation())
    }
}

impl IErc1967Utils for Erc1967Proxy {
    fn get_implementation(&self) -> Address {
        todo!()
    }

    fn upgrade_to_and_call(
        &mut self,
        _new_implementation: Address,
        _data: Bytes,
    ) {
        todo!()
    }

    fn get_admin(&self) -> Address {
        todo!()
    }

    fn change_admin(&mut self, _new_admin: Address) {
        todo!()
    }

    fn get_beacon(&self) -> Address {
        todo!()
    }

    fn upgrade_beacon_to_and_call(
        &mut self,
        _new_beacon: Address,
        _data: Bytes,
    ) {
        todo!()
    }
}
