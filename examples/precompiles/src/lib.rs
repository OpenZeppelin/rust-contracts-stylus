#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B1024, Address, B256};
use openzeppelin_stylus::utils::{
    cryptography::ecdsa,
    precompiles::{self, Precompiles},
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct PrecompilesExample;

#[public]
impl PrecompilesExample {
    fn recover(
        &mut self,
        hash: B256,
        v: u8,
        r: B256,
        s: B256,
    ) -> Result<Address, ecdsa::Error> {
        self.ecrecover(hash, v, r, s)
    }

    fn call_bls12_g1_add(&self, a: Bytes, b: Bytes) -> Result<Bytes, Vec<u8>> {
        let result = self.bls12_g1_add(
            B1024::try_from(a.as_slice()).map_err(|_| {
                precompiles::Error::Bls12G1AddInvalidInput(
                    precompiles::BLS12G1AddInvalidInput {},
                )
            })?,
            B1024::try_from(b.as_slice()).map_err(|_| {
                precompiles::Error::Bls12G1AddInvalidInput(
                    precompiles::BLS12G1AddInvalidInput {},
                )
            })?,
        )?;

        Ok(result)
    }
}
