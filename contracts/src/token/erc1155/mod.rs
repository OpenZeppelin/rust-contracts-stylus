//! Implementation of the [`Erc1155`] token standard.
use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use stylus_sdk::{
    abi::Bytes, alloy_sol_types::sol, call::MethodError, prelude::*,
};

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

/// Required interface of an [`Erc1155`] compliant contract.
pub trait IErc1155 {
    /// The error type associated to this ERC-721 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the number of tokens in ``owner``'s account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    fn balance_of(&self, owner: Address) -> Result<U256, Self::Error>;

    /// xref:ROOT:erc1155.adoc#batch-operations[Batched] version of {balanceOf}.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `accounts` - All account of the tokens' owner.
    /// * `ids` - All token identifiers.
    ///
    /// Requirements:
    ///
    /// * - `accounts` and `ids` must have the same length.
    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Address, Self::Error>;

    /// Grants or revokes permission to `operator` to transfer the caller's
    /// tokens, according to `approved`,
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `approved` - Flag that determines whether or not permission will be
    ///   granted to `operator`. If true, this means `operator` will be allowed
    ///   to manage `msg::sender`'s assets.
    ///
    /// # Errors
    ///
    /// * If `operator` is `Address::ZERO`, then the error
    /// [`Error::InvalidOperator`] is returned.
    ///
    /// # Requirements:
    ///
    /// * The `operator` cannot be the `Address::ZERO`.
    ///
    /// # Events
    ///
    /// Emits an [`ApprovalForAll`] event.
    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error>;

    /// Returns true if `operator` is approved to transfer ``account``'s
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `operator` - Account to be checked.
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;

    /// Transfers a `value` amount of tokens of type `id` from `from` to `to`.
    ///
    /// WARNING: This function can potentially allow a reentrancy attack when
    /// transferring tokens to an untrusted contract, when invoking
    /// [`IERC1155Receiver::on_erc_1155_received`] on the receiver. Ensure to
    /// follow the checks-effects-interactions pattern and consider
    /// employing reentrancy guards when interacting with untrusted
    /// contracts.
    ///
    /// Emits a [`TransferSingle`] event.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `from` is `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If the `from` is not sender, then the error
    /// [`Error::MissingApprovalForAll`] is returned.
    /// If the caller does not have the right to approve, then the error
    /// [`Error::MissingApprovalForAll`] is returned.
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements:
    /// *
    /// * - `to` cannot be the zero address.
    /// * - If the caller is not `from`, it must have been approved to spend
    ///   ``from``'s tokens via [`Self::set_approval_for_all`].
    /// * - `from` must have a balance of tokens of type `id` of at least
    ///   `value` amount.
    /// * - If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_received`] and return the
    /// acceptance magic value.
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error>;

    /// xref:ROOT:erc1155.adoc#batch-operations[Batched] version of
    /// [`Self::safe_transfer_from`].
    ///
    /// WARNING: This function can potentially allow a reentrancy attack when
    /// transferring tokens to an untrusted contract, when invoking
    /// [`IERC1155Receiver::on_erc_1155_batch_received`] on the receiver. Ensure
    /// to follow the checks-effects-interactions pattern and consider
    /// employing reentrancy guards when interacting with untrusted
    /// contracts.
    ///
    /// Emits either a [`TransferSingle`] or a [`TransferBatch`] event,
    /// depending on the length of the array arguments.
    ///
    /// * Requirements:
    /// *
    /// * - `ids` and `values` must have the same length.
    /// * - If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] and return the
    /// acceptance magic value.
    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error>;
}
