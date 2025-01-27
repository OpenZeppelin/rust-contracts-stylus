#![allow(missing_docs)]
#![cfg_attr(coverage_nightly, coverage(off))]

use alloc::vec;

use stylus_sdk::stylus_proc::sol_interface;

sol_interface! {
    /// [`super::Erc721`] token receiver interface.
    ///
    /// Interface for any contract that wants to support `safe_transfers`
    /// from [`super::Erc721`] asset contracts.
    interface IERC721Receiver {
        /// Whenever an [`super::Erc721`] `token_id` token is transferred
        /// to this contract via [`super::IErc721::safe_transfer_from`].
        ///
        /// It must return its function selector to confirm the token transfer.
        /// If any other value is returned or the interface is not implemented
        /// by the recipient, the transfer will be reverted.
        ///
        /// # Arguments
        ///
        /// * `operator` - Account of the operator.
        /// * `from` - Account of the sender.
        /// * `token_id` - Token id as a number.
        /// * `data` - Additional data with no specified format, sent in call.
        #[allow(missing_docs)]
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}
