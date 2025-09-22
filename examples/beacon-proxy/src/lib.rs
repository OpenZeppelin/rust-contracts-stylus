#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![allow(clippy::needless_pass_by_value)]

extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_stylus::proxy::{beacon::proxy::BeaconProxy, erc1967, IProxy};
use stylus_sdk::{
    abi::Bytes, alloy_primitives::Address, prelude::*, ArbResult,
};

#[entrypoint]
#[storage]
struct BeaconProxyExample {
    beacon_proxy: BeaconProxy,
}

#[public]
impl BeaconProxyExample {
    #[constructor]
    pub fn constructor(
        &mut self,
        beacon: Address,
        data: Bytes,
    ) -> Result<(), erc1967::utils::Error> {
        self.beacon_proxy.constructor(beacon, &data)
    }

    fn implementation(&self) -> Result<Address, Vec<u8>> {
        IProxy::implementation(self)
    }

    fn get_beacon(&self) -> Address {
        self.beacon_proxy.get_beacon()
    }

    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        unsafe { self.do_fallback(calldata) }
    }
}

unsafe impl IProxy for BeaconProxyExample {
    fn implementation(&self) -> Result<Address, Vec<u8>> {
        self.beacon_proxy.implementation()
    }
}
