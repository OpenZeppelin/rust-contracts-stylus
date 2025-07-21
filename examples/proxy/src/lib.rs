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

    fn implementation(&self) -> Result<Address, Vec<u8>> {
        IProxy::implementation(self)
    }

    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        unsafe { self.do_fallback(calldata) }
    }
}

unsafe impl IProxy for ProxyExample {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        Ok(self.implementation.get())
    }
}
