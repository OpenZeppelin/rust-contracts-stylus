//! Module with an interface required for smart contract in order to receive
//! ERC-721 token transfers.
#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]
use alloc::vec;

use stylus_sdk::prelude::sol_interface;

sol_interface! {
    /// [`super::Erc721`] token receiver interface.
    ///
    /// Interface for any contract that wants to support safe transfers from
    /// [`super::Erc721`] asset contracts.
    interface IERC721Receiver {
        /// This function is called whenever an [`super::Erc721`] `token_id`
        /// token is transferred to this contract via
        /// [`super::IErc721::safe_transfer_from`].
        ///
        /// It must return its function selector to confirm the token transfer.
        /// If any other value is returned or the interface is not implemented
        /// by the recipient, the transfer will be reverted.
        ///
        /// NOTE: To accept the transfer, this must return
        /// [`super::RECEIVER_FN_SELECTOR`], or its own function selector.
        ///
        /// # Arguments
        ///
        /// * `operator` - Account of the operator.
        /// * `from` - Account of the sender.
        /// * `token_id` - Token id as a number.
        /// * `data` - Additional data with no specified format.
        #[allow(missing_docs)]
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}
