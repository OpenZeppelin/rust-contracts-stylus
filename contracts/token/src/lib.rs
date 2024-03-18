#![doc = include_str!("../README.md")]
#![warn(missing_docs, unreachable_pub, rust_2021_compatibility)]
#![warn(clippy::all, clippy::pedantic)]
#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

pub mod erc20;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
