#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use stylus_sdk::prelude::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct CryptoExample {}
}

#[external]
impl CryptoExample {
    fn test(&self) -> u64 {
        0
    }
}
