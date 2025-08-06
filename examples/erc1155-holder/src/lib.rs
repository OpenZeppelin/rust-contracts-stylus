#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use openzeppelin_stylus::{
    token::erc1155::{receiver::IErc1155Receiver, utils::Erc1155Holder},
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{aliases::B32, Address, U256},
    prelude::*,
};

#[entrypoint]
#[storage]
struct Erc1155HolderExample {
    holder: Erc1155Holder,
}

#[public]
#[implements(IErc1155Receiver, IErc165)]
impl Erc1155HolderExample {}

#[public]
impl IErc1155Receiver for Erc1155HolderExample {
    #[selector(name = "onERC1155Received")]
    fn on_erc1155_received(
        &mut self,
        operator: Address,
        from: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        self.holder.on_erc1155_received(operator, from, id, value, data)
    }

    #[selector(name = "onERC1155BatchReceived")]
    fn on_erc1155_batch_received(
        &mut self,
        operator: Address,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        self.holder.on_erc1155_batch_received(operator, from, ids, values, data)
    }
}

#[public]
impl IErc165 for Erc1155HolderExample {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.holder.supports_interface(interface_id)
    }
}
