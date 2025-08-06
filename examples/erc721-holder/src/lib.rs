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
#[implements(IErc721Receiver)]
impl Erc721HolderExample {}

#[public]
impl IErc721Receiver for Erc721HolderExample {
    #[selector(name = "onERC721Received")]
    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        self.holder.on_erc721_received(operator, from, token_id, data)
    }
}
