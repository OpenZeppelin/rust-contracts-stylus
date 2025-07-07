#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::proxy::IProxy;
use stylus_sdk::{
    alloy_primitives::Address, prelude::*, storage::StorageAddress, ArbResult,
};

#[entrypoint]
#[storage]
struct ProxyExample {
    implementation: StorageAddress,
}

#[public]
impl ProxyExample {
    #[constructor]
    pub fn constructor(&mut self, implementation: Address) {
        self.implementation.set(implementation);
    }

    fn implementation(&self) -> Address {
        IProxy::implementation(self)
    }

    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        self.do_fallback(calldata)
    }
}

impl IProxy for ProxyExample {
    fn implementation(&self) -> Address {
        self.implementation.get()
    }
}
