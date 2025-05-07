#![cfg_attr(not(test), no_main)]
extern crate alloc;

use alloy_primitives::{Address, B256};
use openzeppelin_stylus::utils::cryptography::ecdsa;
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
        ecdsa::recover(self, hash, v, r, s)
    }
}
