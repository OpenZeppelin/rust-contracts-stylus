//! Module with an interface required for smart contract in order to receive
//! ERC-1155 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{abi::Bytes, function_selector, prelude::*};

use crate::utils::introspection::erc165::IErc165;

/// The expected value returned from [`IErc1155Receiver::on_erc1155_received`].
pub const SINGLE_TRANSFER_FN_SELECTOR: B32 = B32::new(function_selector!(
    "onERC1155Received",
    Address,
    Address,
    U256,
    U256,
    Bytes
));

/// The expected value returned from
/// [`IErc1155Receiver::on_erc1155_batch_received`].
pub const BATCH_TRANSFER_FN_SELECTOR: B32 = B32::new(function_selector!(
    "onERC1155BatchReceived",
    Address,
    Address,
    Vec<U256>,
    Vec<U256>,
    Bytes
));

sol_interface! {
    /// [`super::Erc1155`] token receiver Solidity interface.
    ///
    /// Check [`super::IErc1155Receiver`] trait for more details.
    interface IErc1155ReceiverInterface {
        /// See [`super::IErc1155Receiver::on_erc1155_received`].
        #[allow(missing_docs)]
        function onERC1155Received(
            address operator,
            address from,
            uint256 id,
            uint256 value,
            bytes calldata data
        ) external returns (bytes4);

        /// See [`super::IErc1155Receiver::on_erc1155_batch_received`].
        #[allow(missing_docs)]
        function onERC1155BatchReceived(
            address operator,
            address from,
            uint256[] calldata ids,
            uint256[] calldata values,
            bytes calldata data
        ) external returns (bytes4);
    }
}

/// Interface that must be implemented by smart contracts in order to receive
/// ERC-1155 token transfers.
#[interface_id]
#[public]
pub trait IErc1155Receiver: IErc165 {
    /// Handles the receipt of a single ERC-1155 token type. This function
    /// is called at the end of [`super::IErc1155::safe_transfer_from`] after
    /// the balance has been updated.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - The address which initiated the transfer.
    /// * `from` - The address which previously owned the token.
    /// * `id` - The ID of the token being transferred.
    /// * `value` - The amount of tokens being transferred.
    /// * `data` - Additional data with no specified format.
    ///
    /// # Errors
    ///
    /// * May return a custom error.
    #[selector(name = "onERC1155Received")]
    fn on_erc1155_received(
        &mut self,
        operator: Address,
        from: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<B32, Vec<u8>>;

    /// Handles the receipt of multiple ERC-1155 token types. This function
    /// is called at the end of
    /// [`super::IErc1155::safe_batch_transfer_from`] after the balances
    /// have been updated.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - The address which initiated the batch transfer.
    /// * `from` - The address which previously owned the token.
    /// * `ids` - An array containing ids of each token being transferred (order
    ///   and length must match `values` array).
    /// * `values` - An array containing amounts of each token being transferred
    ///   (order and length must match `ids` array).
    /// * `data` - Additional data with no specified format.
    ///
    /// # Errors
    ///
    /// * May return a custom error.
    #[selector(name = "onERC1155BatchReceived")]
    fn on_erc1155_batch_received(
        &mut self,
        operator: Address,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<B32, Vec<u8>>;
}
