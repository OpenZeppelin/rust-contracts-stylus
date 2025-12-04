//! Interface required for smart contract in order to receive
//! ERC-721 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{abi::Bytes, function_selector};

/// The expected value returned from [`IErc721Receiver::on_erc721_received`].
pub const RECEIVER_FN_SELECTOR: B32 = B32::new(function_selector!(
    "onERC721Received",
    Address,
    Address,
    U256,
    Bytes,
));

/// [`super::IErc721`] token receiver trait.
///
/// Interface for any contract that wants to support
/// [`super::IErc721::safe_transfer_from`]
/// and [`super::IErc721::safe_transfer_from_with_data`] from ERC-721 asset
/// contracts.
#[interface_id]
pub trait IErc721Receiver {
    /// This function is called whenever an [`super::Erc721`] `token_id`
    /// token is transferred to this contract via
    /// [`super::IErc721::safe_transfer_from`] or
    /// [`super::IErc721::safe_transfer_from_with_data`].
    ///
    /// It must return its its Solidity selector to confirm the token transfer.
    /// If any other value is returned or the interface is not implemented
    /// by the recipient, the transfer will be reverted.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account of the operator.
    /// * `from` - Account of the sender.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format.
    ///
    /// # Errors
    ///
    /// * May return a custom error.
    fn on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>>;
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
pub(crate) mod tests {
    use stylus_sdk::prelude::*;

    use super::*;
    use crate::utils::introspection::erc165::IErc165;

    /// ERC-721 receiver that returns the wrong selector.
    #[storage]
    pub(crate) struct BadSelectorReceiver721;

    unsafe impl TopLevelStorage for BadSelectorReceiver721 {}

    #[public]
    #[implements(IErc721Receiver, IErc165)]
    impl BadSelectorReceiver721 {}

    #[public]
    impl IErc721Receiver for BadSelectorReceiver721 {
        #[selector(name = "onERC721Received")]
        fn on_erc721_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _token_id: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Ok(B32::ZERO) // wrong selector -> must be rejected
        }
    }

    #[public]
    impl IErc165 for BadSelectorReceiver721 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc721Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    /// ERC-721 receiver that reverts.
    #[storage]
    pub(crate) struct RevertingReceiver721;

    unsafe impl TopLevelStorage for RevertingReceiver721 {}

    #[public]
    #[implements(IErc721Receiver, IErc165)]
    impl RevertingReceiver721 {}

    #[public]
    impl IErc721Receiver for RevertingReceiver721 {
        #[selector(name = "onERC721Received")]
        fn on_erc721_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _token_id: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Err("Receiver rejected".into())
        }
    }

    #[public]
    impl IErc165 for RevertingReceiver721 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc721Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }

    /// ERC-721 receiver that reverts with an empty reason.
    #[storage]
    pub(crate) struct EmptyReasonReceiver721;

    unsafe impl TopLevelStorage for EmptyReasonReceiver721 {}

    #[public]
    #[implements(IErc721Receiver, IErc165)]
    impl EmptyReasonReceiver721 {}

    #[public]
    impl IErc721Receiver for EmptyReasonReceiver721 {
        #[selector(name = "onERC721Received")]
        fn on_erc721_received(
            &mut self,
            _operator: Address,
            _from: Address,
            _token_id: U256,
            _data: Bytes,
        ) -> Result<B32, Vec<u8>> {
            Err(Vec::new())
        }
    }

    #[public]
    impl IErc165 for EmptyReasonReceiver721 {
        fn supports_interface(&self, interface_id: B32) -> bool {
            <Self as IErc721Receiver>::interface_id() == interface_id
                || <Self as IErc165>::interface_id() == interface_id
        }
    }
}
