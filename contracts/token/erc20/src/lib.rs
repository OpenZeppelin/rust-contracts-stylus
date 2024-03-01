#![cfg_attr(not(feature = "export-abi"), no_std)]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

mod erc20;

use erc20::{Erc20, IErc20Metadata};
use stylus_sdk::stylus_proc::{entrypoint, external, sol_storage};

struct TokenMetadata;

pub const TOKEN_NAME: &'static str = "Token";
pub const TOKEN_SYMBOL: &'static str = "TKN";
pub const TOKEN_DECIMALS: u8 = 6;

impl IErc20Metadata for TokenMetadata {
    const NAME: &'static str = TOKEN_NAME;
    const SYMBOL: &'static str = TOKEN_SYMBOL;
    const DECIMALS: u8 = TOKEN_DECIMALS;
}

sol_storage! {
    #[entrypoint]
    struct Token {
        #[borrow]
        Erc20<TokenMetadata> erc20;
    }
}

#[external]
#[inherit(Erc20<TokenMetadata>)]
impl Token {}
