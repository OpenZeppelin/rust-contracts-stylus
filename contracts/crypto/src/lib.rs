#![cfg_attr(not(feature = "export-abi"), no_main, no_std)]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

mod merkle_proof;

use stylus_sdk::stylus_proc::{entrypoint, external, sol_storage};

sol_storage! {
    #[entrypoint]
    struct Dummy {
    }
}

#[external]
impl Dummy {}
