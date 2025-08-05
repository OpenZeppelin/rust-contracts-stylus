#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use openzeppelin_stylus::token::erc721::{
    receiver::IErc721Receiver, utils::Erc721Holder,
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{aliases::B32, Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc721HolderExample {
    holder: Erc721Holder,
}

#[public]
#[implements(IErc721Receiver<Error = Vec<u8>>)]
impl Erc721HolderExample {
    #[constructor]
    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }
}

#[public]
impl IErc721Receiver for Erc721HolderExample {
    type Error = Vec<u8>;

    #[selector(name = "onERC721Received")]
    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<B32, Self::Error> {
        self.holder.on_erc721_received(operator, from, token_id, data)
    }
}
