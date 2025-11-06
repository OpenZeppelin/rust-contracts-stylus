//! Implementation of the [`IErc1155Receiver`] trait.
use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use stylus_sdk::{abi::Bytes, prelude::*};

use crate::{
    token::erc1155::receiver::{
        IErc1155Receiver, BATCH_TRANSFER_FN_SELECTOR,
        SINGLE_TRANSFER_FN_SELECTOR,
    },
    utils::introspection::erc165::IErc165,
};

/// Simple implementation of [`IErc1155Receiver`] that will allow a contract to
/// hold ERC-1155 tokens.
#[storage]
pub struct Erc1155Holder;

#[public]
#[implements(IErc1155Receiver, IErc165)]
impl Erc1155Holder {}

#[public]
impl IErc1155Receiver for Erc1155Holder {
    #[selector(name = "onERC1155Received")]
    fn on_erc1155_received(
        &mut self,
        _operator: Address,
        _from: Address,
        _id: U256,
        _value: U256,
        _data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        Ok(SINGLE_TRANSFER_FN_SELECTOR)
    }

    #[selector(name = "onERC1155BatchReceived")]
    fn on_erc1155_batch_received(
        &mut self,
        _operator: Address,
        _from: Address,
        _ids: Vec<U256>,
        _values: Vec<U256>,
        _data: Bytes,
    ) -> Result<B32, Vec<u8>> {
        Ok(BATCH_TRANSFER_FN_SELECTOR)
    }
}

#[public]
impl IErc165 for Erc1155Holder {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc1155Receiver>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::Contract;

    use super::*;

    unsafe impl TopLevelStorage for Erc1155Holder {}

    #[motsu::test]
    fn holder_returns_proper_selectors(
        contract: Contract<Erc1155Holder>,
        alice: Address,
    ) {
        assert_eq!(
            contract.sender(alice).on_erc1155_received(
                alice,
                alice,
                U256::ONE,
                U256::ONE,
                vec![].into()
            ),
            Ok(SINGLE_TRANSFER_FN_SELECTOR)
        );

        assert_eq!(
            contract.sender(alice).on_erc1155_batch_received(
                alice,
                alice,
                vec![U256::ONE],
                vec![U256::ONE],
                vec![].into()
            ),
            Ok(BATCH_TRANSFER_FN_SELECTOR)
        );
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc1155Holder>, alice: Address) {
        assert!(contract.sender(alice).supports_interface(
            <Erc1155Holder as IErc1155Receiver>::interface_id()
        ));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc1155Holder as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
}
