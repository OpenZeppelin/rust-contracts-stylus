#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::needless_pass_by_value)]

extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::{
    proxy::{
        erc1967::{self},
        IProxy,
    },
    utils::address::AddressUtils,
};
use stylus_sdk::{
    abi::Bytes, alloy_primitives::Address, prelude::*, storage::StorageAddress,
    ArbResult,
};

#[entrypoint]
#[storage]
struct Erc1967InvalidExample {
    implementation: StorageAddress,
    address_utils: AddressUtils,
}

#[public]
impl Erc1967InvalidExample {
    #[constructor]
    pub fn constructor(
        &mut self,
        implementation: Address,
        data: Bytes,
    ) -> Result<(), erc1967::utils::Error> {
        self.implementation.set(implementation);
        // "forget" to set the implementation address at the appropriate
        // implementation slot
        self.address_utils
            .function_delegate_call(implementation, data.as_ref())?;
        Ok(())
    }

    fn implementation(&self) -> Result<Address, Vec<u8>> {
        IProxy::implementation(self)
    }

    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        unsafe { self.do_fallback(calldata) }
    }
}

unsafe impl IProxy for Erc1967InvalidExample {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        Ok(self.implementation.get())
    }
}
