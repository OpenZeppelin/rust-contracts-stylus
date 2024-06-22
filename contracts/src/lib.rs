#![doc = include_str!("../../README.md")]
#![allow(clippy::pub_underscore_fields, clippy::module_name_repetitions)]
#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[cfg(any(feature = "std", feature = "access"))]
pub mod access;
pub mod token;
pub mod utils;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
