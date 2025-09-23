//! Module with an interface required for smart contract in order to receive
//! ERC-721 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{abi::Bytes, function_selector, prelude::*};

/// The expected value returned from [`IErc721Receiver::on_erc721_received`].
pub const RECEIVER_FN_SELECTOR: B32 = B32::new(function_selector!(
    "onERC721Received",
    Address,
    Address,
    U256,
    Bytes,
));

sol_interface! {
    /// [`super::Erc721`] token receiver Solidity interface.
    ///
    /// Check [`super::IErc721Receiver`] trait for more details.
    interface IErc721ReceiverInterface {
        /// See [`super::IErc721Receiver::on_erc721_received`].
        #[allow(missing_docs)]
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}

/// [`super::IErc721`] token receiver trait.
///
/// Interface for any contract that wants to support
/// [`super::IErc721::safe_transfer_from`]
/// and [`super::IErc721::safe_transfer_from_with_data`] from ERC-721 asset
/// contracts.
#[interface_id]
#[public]
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
