#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256};
use openzeppelin_stylus::utils::{
    cryptography::ecdsa, precompiles::Precompiles,
};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct ECDSAExample;

#[public]
impl ECDSAExample {
    fn recover(
        &mut self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, ecdsa::Error> {
        self.ecrecover(hash, v, r, s)
    }
}
