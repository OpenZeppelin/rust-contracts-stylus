//! Implementation of the [`IErc721Receiver`] trait.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::{abi::Bytes, function_selector, prelude::*};

use crate::token::erc721::receiver::{IErc721Receiver, RECEIVER_FN_SELECTOR};

/// Default implementation of the [`IErc721Receiver`] trait.
#[storage]
pub struct Erc721Holder;

#[public]
#[implements(IErc721Receiver<Error = Vec<u8>>)]
impl Erc721Holder {}

#[public]
impl IErc721Receiver for Erc721Holder {
    type Error = Vec<u8>;

    #[selector(name = "onERC721Received")]
    fn on_erc721_received(
        &mut self,
        _operator: Address,
        _from: Address,
        _token_id: U256,
        _data: Bytes,
    ) -> Result<FixedBytes<4>, Self::Error> {
        Ok(RECEIVER_FN_SELECTOR)
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::Contract;

    use super::*;

    unsafe impl TopLevelStorage for Erc721Holder {}

    #[motsu::test]
    fn holder_returns_proper_selector(
        contract: Contract<Erc721Holder>,
        alice: Address,
    ) {
        assert_eq!(
            contract.sender(alice).on_erc721_received(
                alice,
                alice,
                U256::from(1),
                vec![].into()
            ),
            Ok(RECEIVER_FN_SELECTOR)
        );
    }
}
