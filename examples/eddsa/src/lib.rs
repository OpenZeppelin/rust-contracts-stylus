#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;

use openzeppelin_crypto::eddsa::{SecretKey, SigningKey};
use stylus_sdk::{abi::Bytes, prelude::*};

#[entrypoint]
#[storage]
struct EddsaExample;

#[public]
impl EddsaExample {
    /// Signing is pointless onchain, since the `secret_key` should be always
    /// private.
    /// The purpose of this example is to:
    ///
    /// * Show how EDDSA can be configured.
    /// * Check that it's deployable.
    fn sign(
        &mut self,
        secret_key: alloy_primitives::U256,
        message: Bytes,
    ) -> Bytes {
        let secret_key: SecretKey = secret_key.to_le_bytes();

        let signing_key = SigningKey::from_bytes(&secret_key);
        let signature = signing_key.sign(&message);

        signature.to_bytes().to_vec().into()
    }
}
