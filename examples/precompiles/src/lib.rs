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
struct PrecompilesExample;

#[public]
impl PrecompilesExample {
    fn ec_recover_example(
        &mut self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, ecdsa::Error> {
        self.ec_recover(hash, v, r, s)
    }

    fn p256_verify_example(
        &self,
        hash: B256,
        r: B256,
        s: B256,
        x: B256,
        y: B256,
    ) -> bool {
        self.p256_verify(hash, r, s, x, y)
    }
}
