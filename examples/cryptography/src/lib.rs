#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, FixedBytes};
use openzeppelin_stylus::utils::cryptography::ecdsa;
use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct CryptoExample {}
}

#[external]
impl CryptoExample {
    // #[selector(name = "recover")]
    pub fn recover_from_signature(
        &mut self,
        hash: FixedBytes<32>,
        signature: Vec<u8>,
    ) -> Result<Address, Vec<u8>> {
        let signer =
            ecdsa::recover_from_signature(self, hash, signature.into())?;
        Ok(signer)
    }

    // #[selector(name = "recover")]
    pub fn recover_from_r_vs(
        &mut self,
        hash: FixedBytes<32>,
        r: FixedBytes<32>,
        vs: FixedBytes<32>,
    ) -> Result<Address, Vec<u8>> {
        let signer = ecdsa::recover_from_r_vs(self, hash, r, vs)?;
        Ok(signer)
    }

    // #[selector(name = "recover")]
    pub fn recover(
        &mut self,
        hash: FixedBytes<32>,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<Address, Vec<u8>> {
        let signer = ecdsa::recover(self, hash, v, r, s)?;
        Ok(signer)
    }
}
