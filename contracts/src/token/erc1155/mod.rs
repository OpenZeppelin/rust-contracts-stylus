//! Implementation of the [`Erc1155`] token standard.

use stylus_sdk::{alloy_sol_types::sol, call::MethodError, prelude::*};

pub mod extensions;

sol! {
    /// Emitted when `value` amount of tokens of type `id` are transferred from `from` to `to` by `operator`.
    #[allow(missing_docs)]
    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 id,
        uint256 value
    );

    /// Equivalent to multiple {TransferSingle} events, where `operator`.
    /// `from` and `to` are the same for all transfers.
    #[allow(missing_docs)]
    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] values
    );

    /// Emitted when `account` grants or revokes permission to `operator` to transfer their tokens, according to
    /// `approved`.
    #[allow(missing_docs)]
    event ApprovalForAll(address indexed account, address indexed operator, bool approved);

    /// Emitted when the URI for token type `id` changes to `value`, if it is a non-programmatic URI.
    ///
    /// If an {URI} event was emitted for `id`, the [standard]
    /// (https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions[guarantees]) that `value` will equal the value
    /// returned by [`IERC1155MetadataURI-uri`].
    #[allow(missing_docs)]
    event URI(string value, uint256 indexed id);
}

sol! {
    /// Indicates an error related to the current `balance` of a `sender`. Used
    /// in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    /// * `balance` - Current balance for the interacting account.
    /// * `needed` - Minimum amount required to perform a transfer.
    /// * `tokenId` - Identifier number of a token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 tokenId);

    /// Indicates a failure with the token `sender`. Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidSender(address sender);

    /// Indicates a failure with the token `receiver`. Used in transfers.
    ///
    /// * `receiver` - Address to which tokens are being transferred.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidReceiver(address receiver);

    /// Indicates a failure with the `operator`’s approval. Used
    /// in transfers.
    ///
    /// * `operator` - Address that may be allowed to operate on tokens without being their owner.
    /// * `owner` - Address of the current owner of a token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155MissingApprovalForAll(address operator, address owner);

    /// Indicates a failure with the `approver` of a token to be approved. Used
    ///  in approvals.
    ///
    /// * `approver` - Address initiating an approval operation.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidApprover(address approver);

    /// Indicates a failure with the `operator` to be approved. Used
    ///  in approvals.
    ///
    /// * `operator` - Address that may be allowed to operate on tokens without being their owner.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidOperator(address operator);

    /// Indicates an array length mismatch between ids and values in a safeBatchTransferFrom operation.
    /// Used in batch transfers.
    ///
    /// * `idsLength` - Length of the array of token identifiers.
    /// * `valuesLength` - Length of the array of token amounts.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidArrayLength(uint256 idsLength, uint256 valuesLength);
}

/// An [`Erc1155`] error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the current `balance` of `sender`. Used
    /// in transfers.
    InsufficientBalance(ERC1155InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(ERC1155InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(ERC1155InvalidReceiver),
    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    MissingApprovalForAll(ERC1155MissingApprovalForAll),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(ERC1155InvalidApprover),
    /// Indicates a failure with the `operator` to be approved. Used in
    /// approvals.
    InvalidOperator(ERC1155InvalidOperator),
    /// Indicates an array length mismatch between ids and values in a
    /// safeBatchTransferFrom operation. Used in batch transfers.
    InvalidArrayLength(ERC1155InvalidArrayLength),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

sol_interface! {
    interface IERC1155Receiver {

        /// Handles the receipt of a single ERC-1155 token type. This function is
        /// called at the end of a `safeTransferFrom` after the balance has been updated.
        ///
        /// NOTE: To accept the transfer, this must return
        /// `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))`
        /// (i.e. 0xf23a6e61, or its own function selector).
        ///
        /// * `operator` - The address which initiated the transfer (i.e. msg.sender)
        /// * `from` - The address which previously owned the token
        /// * `id` - The ID of the token being transferred
        /// * `value` - The amount of tokens being transferred
        /// * `data` - Additional data with no specified format
        /// Return `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))` if transfer is allowed
        #[allow(missing_docs)]
        function onERC1155Received(
            address operator,
            address from,
            uint256 id,
            uint256 value,
            bytes calldata data
        ) external returns (bytes4);

        /// Handles the receipt of a multiple ERC-1155 token types. This function
        /// is called at the end of a `safeBatchTransferFrom` after the balances have
        /// been updated.
        ///
        /// NOTE: To accept the transfer(s), this must return
        /// `bytes4(keccak256("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"))`
        /// (i.e. 0xbc197c81, or its own function selector).
        ///
        /// * `operator` - The address which initiated the batch transfer (i.e. msg.sender)
        /// * `from` - The address which previously owned the token
        /// * `ids` - An array containing ids of each token being transferred (order and length must match values array)
        /// * `values` - An array containing amounts of each token being transferred (order and length must match ids array)
        /// * `data` - Additional data with no specified format
        /// * Return `bytes4(keccak256("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"))` if transfer is allowed
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

sol_storage! {
    /// State of an [`Erc1155`] token.
    pub struct Erc1155 {
        /// Maps users to balances.
        mapping(uint256 => mapping(address => uint256)) _balances;
        /// Maps owners to a mapping of operator approvals.
        mapping(address => mapping(address => bool)) _operator_approvals;
    }
}
