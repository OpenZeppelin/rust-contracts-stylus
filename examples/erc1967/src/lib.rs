#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::proxy::{
    erc1967::{self, Erc1967Proxy},
    IProxy,
};
use stylus_sdk::{
    abi::Bytes, alloy_primitives::Address, prelude::*, ArbResult,
};

#[entrypoint]
#[storage]
struct Erc1967Example {
    erc1967: Erc1967Proxy,
}

#[public]
impl Erc1967Example {
    #[constructor]
    pub fn constructor(
        &mut self,
        implementation: Address,
        data: Bytes,
    ) -> Result<(), erc1967::Error> {
        self.erc1967.constructor(implementation, data)
    }

    fn implementation(&self) -> Address {
        IProxy::implementation(self)
    }

    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        self.do_fallback(calldata)
    }
}

impl IProxy for Erc1967Example {
    fn implementation(&self) -> Address {
        self.erc1967.implementation()
    }
}
