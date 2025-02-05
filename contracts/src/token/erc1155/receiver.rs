//! Module with an interface required for smart contract in order to receive
//! ERC-1155 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::vec;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    /// [`super::Erc1155`] token receiver interface.
    ///
    /// Interface for any contract that wants to support safe transfers from
    /// [`super::Erc1155`] asset contracts.
    interface IERC1155Receiver {
        /// Handles the receipt of a single ERC-1155 token type. This function
        /// is called at the end of [`super::IErc1155::safe_transfer_from`]
        /// after the balance has been updated.
        ///
        /// NOTE: To accept the transfer, this must return
        /// [`super::SINGLE_TRANSFER_FN_SELECTOR`], or its own function
        /// selector.
        ///
        /// # Arguments
        ///
        /// # Arguments
        ///
        /// * `operator` - The address which initiated the transfer.
        /// * `from` - The address which previously owned the token.
        /// * `id` - The ID of the token being transferred.
        /// * `value` - The amount of tokens being transferred.
        /// * `data` - Additional data with no specified format.
        #[allow(missing_docs)]
        function onERC1155Received(
            address operator,
            address from,
            uint256 id,
            uint256 value,
            bytes calldata data
        ) external returns (bytes4);

        /// Handles the receipt of multiple ERC-1155 token types. This function
        /// is called at the end of
        /// [`super::IErc1155::safe_batch_transfer_from`] after the balances
        /// have been updated.
        ///
        /// NOTE: To accept the transfer(s), this must return
        /// [`super::BATCH_TRANSFER_FN_SELECTOR`], or its own function selector.
        ///
        /// # Arguments
        ///
        /// # Arguments
        ///
        /// * `operator` - The address which initiated the batch transfer.
        /// * `from` - The address which previously owned the token.
        /// * `ids` - An array containing ids of each token being transferred
        ///   (order and length must match `values` array).
        /// * `values` - An array containing amounts of each token being
        ///   transferred (order and length must match `ids` array).
        /// * `data` - Additional data with no specified format.
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
