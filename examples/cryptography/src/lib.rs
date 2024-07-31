#![cfg_attr(not(test), no_std, no_main)]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use crypto::IEIP712;
use stylus_sdk::{
    block, contract,
    prelude::{entrypoint, external, sol_storage},
};

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

sol_storage! {
    struct EIP712 {}
}

#[external]
impl IEIP712 for EIP712 {
    const NAME: &'static str = "A Name";
    const VERSION: &'static str = "1";

    fn chain_id() -> U256 {
        U256::from(block::chainid())
    }

    fn contract_address() -> Address {
        contract::address()
    }
}

sol_storage! {
    #[entrypoint]
    struct CryptoExample {
        #[borrow]
        EIP712 eip712;
    }
}

#[external]
impl CryptoExample {
    pub fn test(&self) -> Result<u8, Vec<u8>> {
        Ok(0)
    }
}
