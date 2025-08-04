#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use openzeppelin_stylus::token::erc721::utils::Erc721Holder;
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
struct Erc721HolderExample {
    holder: Erc721Holder,
}

#[public]
impl Erc721HolderExample {
    #[constructor]
    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }
}

// Implement the IErc721Receiver trait by delegating to the Erc721Holder
impl openzeppelin_stylus::token::erc721::receiver::IErc721Receiver for Erc721HolderExample {
    type Error = Vec<u8>;

    fn on_erc721_received(
        &mut self,
        operator: alloy_primitives::Address,
        from: alloy_primitives::Address,
        token_id: alloy_primitives::U256,
        data: stylus_sdk::abi::Bytes,
    ) -> Result<alloy_primitives::aliases::B32, Self::Error> {
        self.holder.on_erc721_received(operator, from, token_id, data)
    }
}
