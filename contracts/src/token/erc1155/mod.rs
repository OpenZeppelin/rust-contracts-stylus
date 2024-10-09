//! Implementation of the ERC-1155 token standard.
use alloc::{vec, vec::Vec};

use alloy_primitives::{fixed_bytes, Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    alloy_sol_types::sol,
    call::{self, Call, MethodError},
    evm, msg,
    prelude::{public, sol_interface, sol_storage, AddressVM, SolidityError},
    storage::TopLevelStorage,
};

use crate::utils::{
    introspection::erc165::{Erc165, IErc165},
    math::storage::SubAssignUnchecked,
};

/// `bytes4(
///     keccak256(
///         "onERC1155Received(address,address,uint256,uint256,bytes)"
/// ))`
const SINGLE_TRANSFER_FN_SELECTOR: FixedBytes<4> = fixed_bytes!("f23a6e61");

/// `bytes4(
///     keccak256(
///         "onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"
/// ))`
const BATCH_TRANSFER_FN_SELECTOR: FixedBytes<4> = fixed_bytes!("bc197c81");

sol! {
    /// Emitted when `value` amount of tokens of type `id` are
    /// transferred from `from` to `to` by `operator`.
    #[allow(missing_docs)]
    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 id,
        uint256 value
    );

    /// Equivalent to multiple [`TransferSingle`] events, where `operator`
    /// `from` and `to` are the same for all transfers.
    #[allow(missing_docs)]
    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] ids,
        uint256[] values
    );

    /// Emitted when `account` grants or revokes permission to `operator`
    /// to transfer their tokens, according to `approved`.
    #[allow(missing_docs)]
    event ApprovalForAll(
        address indexed account,
        address indexed operator,
        bool approved
    );
}

sol! {
    /// Indicates an error related to the current `balance` of a `sender`.
    /// Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    /// * `balance` - Current balance for the interacting account.
    /// * `needed` - Minimum amount required to perform a transfer.
    /// * `token_id` - Identifier number of a token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InsufficientBalance(
        address sender,
        uint256 balance,
        uint256 needed,
        uint256 token_id
    );

    /// Indicates a failure with the token `sender`.
    /// Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidSender(address sender);

    /// Indicates a failure with the token `receiver`.
    /// Used in transfers.
    ///
    /// * `receiver` - Address to which tokens are being transferred.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidReceiver(address receiver);

    /// Indicates a failure with the `operator`’s approval.
    /// Used in transfers.
    ///
    /// * `operator` - Address that may be allowed to operate on tokens
    ///   without being their owner.
    /// * `owner` - Address of the current owner of a token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155MissingApprovalForAll(address operator, address owner);

    /// Indicates a failure with the `approver` of a token to be approved.
    /// Used in approvals.
    ///
    /// * `approver` - Address initiating an approval operation.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidApprover(address approver);

    /// Indicates a failure with the `operator` to be approved.
    /// Used in approvals.
    ///
    /// * `operator` - Address that may be allowed to operate on tokens
    /// without being their owner.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidOperator(address operator);

    /// Indicates an array length mismatch between token ids and values in a
    /// [`IErc1155::safe_batch_transfer_from`] operation.
    /// Used in batch transfers.
    ///
    /// * `ids_length` - Length of the array of token identifiers.
    /// * `values_length` - Length of the array of token amounts.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InvalidArrayLength(uint256 ids_length, uint256 values_length);
}

/// An [`Erc1155`] error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the current `balance` of `sender`.
    /// Used in transfers.
    InsufficientBalance(ERC1155InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(ERC1155InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(ERC1155InvalidReceiver),
    /// Indicates a failure with the token `receiver`, with the reason
    /// specified by it.
    InvalidReceiverWithReason(call::Error),
    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    MissingApprovalForAll(ERC1155MissingApprovalForAll),
    /// Indicates a failure with the `approver` of a token to be approved.
    /// Used in approvals.
    InvalidApprover(ERC1155InvalidApprover),
    /// Indicates a failure with the `operator` to be approved. Used in
    /// approvals.
    InvalidOperator(ERC1155InvalidOperator),
    /// Indicates an array length mismatch between token ids and values in a
    /// [`Erc1155::safe_batch_transfer_from`] operation.
    /// Used in batch transfers.
    InvalidArrayLength(ERC1155InvalidArrayLength),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

sol_interface! {
    /// Interface that must be implemented by smart contracts
    /// in order to receive ERC-1155 token transfers.
    interface IERC1155Receiver {
        /// Handles the receipt of a single ERC-1155 token type.
        /// This function is called at the end of a
        /// [`IErc1155::safe_batch_transfer_from`]
        /// after the balance has been updated.
        ///
        /// NOTE: To accept the transfer,
        /// this must return [`SINGLE_TRANSFER_FN_SELECTOR`],
        /// or its own function selector.
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

        /// Handles the receipt of a multiple ERC-1155 token types.
        /// This function is called at the end of a
        /// [`IErc1155::safe_batch_transfer_from`]
        /// after the balances have been updated.
        ///
        /// NOTE: To accept the transfer(s),
        /// this must return [`BATCH_TRANSFER_FN_SELECTOR`],
        /// or its own function selector.
        ///
        /// * `operator` - The address which initiated the batch transfer.
        /// * `from` - The address which previously owned the token.
        /// * `ids` - An array containing ids of each token being transferred
        ///   (order and length must match values array).
        /// * `values` - An array containing amounts of each token
        ///   being transferred (order and length must match ids array).
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

sol_storage! {
    /// State of an [`Erc1155`] token.
    pub struct Erc1155 {
        /// Maps users to balances.
        mapping(uint256 => mapping(address => uint256)) _balances;
        /// Maps owners to a mapping of operator approvals.
        mapping(address => mapping(address => bool)) _operator_approvals;
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc1155 {}

/// Data structure to be passed to contract
/// implementing [`IErc1155Receiver`] interface.
struct Erc1155ReceiverData {
    /// Function Selector
    fn_selector: FixedBytes<4>,
    /// Transfer details, either [`Transfer::Single`] or [`Transfer::Batch`].
    transfer: Transfer,
}

impl Erc1155ReceiverData {
    /// Creates a new instance based on transfer details.
    /// Assumes that `ids` is not empty.
    ///
    /// If `ids` array has only 1 element,
    /// it means that it is a [`Transfer::Single`].
    /// If `ids` array has many elements,
    /// it means that it is a [`Transfer::Batch`].
    ///
    /// NOTE: Does not check if `ids` length is equal to `values`.
    ///
    /// # Arguments
    ///
    /// * `ids` - Array of tokens ids being transferred.
    /// * `values` - Array of all amount of tokens being transferred.
    fn new(ids: Vec<U256>, values: Vec<U256>) -> Self {
        if ids.len() == 1 {
            Self::single(ids[0], values[0])
        } else {
            Self::batch(ids, values)
        }
    }

    /// Creates a new instance for a [`Transfer::Single`].
    /// Check [`IErc1155Receiver::on_erc_1155_received`].
    ///
    /// # Arguments
    ///
    /// * `id` - Token id being transferred.
    /// * `value` - Amount of tokens being transferred.
    fn single(id: U256, value: U256) -> Self {
        Self {
            fn_selector: SINGLE_TRANSFER_FN_SELECTOR,
            transfer: Transfer::Single { id, value },
        }
    }

    /// Creates a new instance for a [`Transfer::Batch`].
    /// Check [`IErc1155Receiver::on_erc_1155_batch_received`].
    ///
    /// # Arguments
    ///
    /// * `ids` - Array of tokens ids being transferred.
    /// * `values` - Array of all amount of tokens being transferred.
    fn batch(ids: Vec<U256>, values: Vec<U256>) -> Self {
        Self {
            fn_selector: BATCH_TRANSFER_FN_SELECTOR,
            transfer: Transfer::Batch { ids, values },
        }
    }
}

/// Struct representing token transfer details.
#[derive(Debug, PartialEq)]
enum Transfer {
    /// Transfer of a single token.
    ///
    /// # Attributes
    ///
    /// * `id` - Token id being transferred.
    /// * `value` - Amount of tokens being transferred.
    Single { id: U256, value: U256 },
    /// Batch tokens transfer.
    ///
    /// # Attributes
    ///
    /// * `ids` - Array of tokens ids being transferred.
    /// * `values` - Array of all amount of tokens being transferred.
    Batch { ids: Vec<U256>, values: Vec<U256> },
}

/// Required interface of an [`Erc1155`] compliant contract.
#[interface_id]
pub trait IErc1155 {
    /// The error type associated to this ERC-1155 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the value of tokens of type `id` owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `id` - Token id as a number.
    fn balance_of(&self, account: Address, id: U256) -> U256;

    /// Batched version of [`IErc1155::balance_of`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `accounts` - All account of the tokens' owner.
    /// * `ids` - All token identifiers.
    ///
    /// # Requirements
    ///
    /// * `accounts` and `ids` must have the same length.
    ///
    /// # Errors
    ///
    /// * If the length of `accounts` is not equal to the length of `ids`,
    /// then the error [`Error::InvalidArrayLength`] is returned.
    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, Self::Error>;

    /// Grants or revokes permission to `operator`
    /// to transfer the caller's tokens, according to `approved`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `approved` - Flag that determines whether or not permission will be
    ///   granted to `operator`. If true, this means `operator` will be allowed
    ///   to manage `msg::sender()`'s assets.
    ///
    /// # Errors
    ///
    /// * If `operator` is `Address::ZERO`, then the error
    /// [`Error::InvalidOperator`] is returned.
    ///
    /// # Requirements
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

    /// Returns true if `operator` is approved to transfer `account`'s
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account of the token's owner.
    /// * `operator` - Account to be checked.
    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool;

    /// Transfers a `value` amount of tokens of type `id` from `from` to
    /// `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id as a number.
    /// * `value` - Amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `from` is `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If the `from` is not the caller (`msg::sender()`),
    /// and the caller does not have the right to approve, then the error
    /// [`Error::MissingApprovalForAll`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `to` cannot be the `Address::ZERO`.
    /// * If the caller is not `from`, it must have been approved to spend
    ///   `from`'s tokens via [`IErc1155::set_approval_for_all`].
    /// * `from` must have a balance of tokens of type `id` of at least `value`
    ///   amount.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_received`] and return the
    ///  acceptance magic value.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event.
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error>;

    /// Batched version of [`IErc1155::safe_transfer_from`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all tokens ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `from` is `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If the `from` is not the caller (`msg::sender()`),
    /// and the caller does not have the right to approve, then the error
    /// [`Error::MissingApprovalForAll`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `ids` length is not equal to `values` length, then the error
    /// [`Error::InvalidArrayLength`]
    ///
    /// # Requirements
    ///
    /// * `to` cannot be the `Address::ZERO`.
    /// * If the caller is not `from`, it must have been approved to spend
    ///   `from`'s tokens via [`IErc1155::set_approval_for_all`].
    /// * `from` must have a balance of tokens being transferred of at least
    ///   transferred amount.
    /// * `ids` and `values` must have the same length.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] and return the
    ///   acceptance magic value.
    ///
    /// # Events
    ///
    /// Emits either a [`TransferSingle`] or a [`TransferBatch`] event,
    /// depending on the length of the array arguments.
    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error>;
}

#[public]
impl IErc1155 for Erc1155 {
    type Error = Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self._balances.get(id).get(account)
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, Self::Error> {
        Self::require_equal_arrays(&ids, &accounts)?;

        let balances: Vec<U256> = accounts
            .iter()
            .zip(ids.iter())
            .map(|(account, token_id)| self.balance_of(*account, *token_id))
            .collect();

        Ok(balances)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error> {
        self._set_approval_for_all(msg::sender(), operator, approved)
    }

    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool {
        self._operator_approvals.get(account).get(operator)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.authorize_transfer(from)?;
        self._safe_transfer_from(from, to, id, value, data)
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.authorize_transfer(from)?;
        self._safe_batch_transfer_from(from, to, ids, values, data)
    }
}

impl IErc165 for Erc1155 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc1155>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

impl Erc1155 {
    /// Transfers a `value` amount of tokens of type `ids` from `from` to
    /// `to`. Will mint (or burn) if `from` (or `to`) is the `Address::ZERO`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all tokens ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    ///
    /// # Errors
    ///
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`Error::InsufficientBalance`] is returned.
    ///
    /// NOTE: The ERC-1155 acceptance check is not performed in this function.
    /// See [`Self::_update_with_acceptance_check`] instead.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event if the arrays contain one element, and
    /// [`TransferBatch`] otherwise.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        Self::require_equal_arrays(&ids, &values)?;

        let operator = msg::sender();

        ids.iter().zip(values.iter()).try_for_each(|(&token_id, &value)| {
            self.do_update(from, to, token_id, value)
        })?;

        if ids.len() == 1 {
            let id = ids[0];
            let value = values[0];
            evm::log(TransferSingle { operator, from, to, id, value });
        } else {
            evm::log(TransferBatch { operator, from, to, ids, values });
        }

        Ok(())
    }

    /// Version of [`Self::_update`] that performs the token acceptance check by
    /// calling [`IERC1155Receiver::on_erc_1155_received`] or
    /// [`IERC1155Receiver::on_erc_1155_received`] on the receiver address if it
    /// contains code (eg. is a smart contract at the moment of execution).
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`Error::InsufficientBalance`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event if the arrays contain one element, and
    /// [`TransferBatch`] otherwise.
    fn _update_with_acceptance_check(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        self._update(from, to, ids.clone(), values.clone())?;

        if !to.is_zero() {
            self.do_check_on_erc1155_received(
                msg::sender(),
                from,
                to,
                Erc1155ReceiverData::new(ids, values),
                data.to_vec().into(),
            )?
        }

        Ok(())
    }

    /// Transfers a `value` tokens of token type `id` from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id as a number.
    /// * `value` - Amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is the `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `from` is the `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event.
    fn _safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.do_safe_transfer_from(from, to, vec![id], vec![value], data)
    }

    /// Batched version of [`Self::_safe_transfer_from`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is the `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `from` is the `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`TransferBatch`] event.
    fn _safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        self.do_safe_transfer_from(from, to, ids, values, data)
    }

    /// Creates a `value` amount of tokens of type `id`, and assigns
    /// them to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id.
    /// * `value` - Amount of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event.
    ///
    /// # Panics
    ///
    /// If balance exceeds `U256::MAX`. It may happen during `mint` operation.
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.do_mint(to, vec![id], vec![value], data)
    }

    /// Batched version of [`Self::_mint`].
    ///
    /// # Requirements
    ///
    /// * `to` cannot be the `Address::ZERO`.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_received`] and return the acceptance
    ///   magic value.
    ///
    /// # Events
    ///
    /// Emits a [`TransferBatch`] event.
    ///
    /// # Panics
    ///
    /// If balance exceeds `U256::MAX`. It may happen during `mint` operation.
    pub fn _mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        self.do_mint(to, ids, values, data)
    }

    /// Destroys a `value` amount of tokens of type `id` from `from`
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event.
    ///
    /// # Errors
    ///
    /// If `from` is the Address::ZERO, then the error
    /// [`Error::InvalidSender`] is returned.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the `Address::ZERO`.
    /// * `from` must have at least `value` amount of tokens of type `id`.
    fn _burn(
        &mut self,
        from: Address,
        id: U256,
        value: U256,
    ) -> Result<(), Error> {
        self.do_burn(from, vec![id], vec![value])
    }

    /// Batched version of [`Self::_burn`].
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event.
    ///
    /// # Errors
    ///
    /// If `from` is the Address::ZERO, then the error
    /// [`Error::InvalidSender`] is returned.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the `Address::ZERO`.
    /// * `from` must have at least `value` amount of tokens of type `id`.
    /// * `ids` and `values` must have the same length.
    fn _burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        self.do_burn(from, ids, values)
    }

    /// Approve `operator` to operate on all of `owner` tokens.
    ///
    /// Emits an [`ApprovalForAll`] event.
    ///
    /// # Requirements
    ///
    /// * `operator` cannot be the `Address::ZERO`.
    ///
    /// # Errors
    ///
    /// If `operator` is the `Address::ZERO`, then the error
    /// [`Error::InvalidOperator`] is returned.
    fn _set_approval_for_all(
        &mut self,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        if operator.is_zero() {
            return Err(Error::InvalidOperator(ERC1155InvalidOperator {
                operator,
            }));
        }
        self._operator_approvals.setter(owner).setter(operator).set(approved);
        evm::log(ApprovalForAll { account: owner, operator, approved });
        Ok(())
    }
}

impl Erc1155 {
    fn do_check_on_erc1155_received(
        &mut self,
        operator: Address,
        from: Address,
        to: Address,
        details: Erc1155ReceiverData,
        data: alloy_primitives::Bytes,
    ) -> Result<(), Error> {
        if !to.has_code() {
            return Ok(());
        }

        let receiver = IERC1155Receiver::new(to);
        let call = Call::new_in(self);
        let result = match details.transfer {
            Transfer::Single { id, value } => receiver
                .on_erc_1155_received(call, operator, from, id, value, data),

            Transfer::Batch { ids, values } => receiver
                .on_erc_1155_batch_received(
                    call, operator, from, ids, values, data,
                ),
        };

        let id = match result {
            Ok(id) => id,
            Err(e) => {
                if let call::Error::Revert(ref reason) = e {
                    if reason.len() > 0 {
                        // Non-IERC1155Receiver implementer.
                        return Err(Error::InvalidReceiverWithReason(e));
                    }
                }

                return Err(ERC1155InvalidReceiver { receiver: to }.into());
            }
        };

        // Token rejected.
        if id != details.fn_selector {
            return Err(ERC1155InvalidReceiver { receiver: to }.into());
        }

        Ok(())
    }

    fn do_mint(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver: to,
            }));
        }
        self._update_with_acceptance_check(
            Address::ZERO,
            to,
            ids,
            values,
            data,
        )?;
        Ok(())
    }

    fn do_burn(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC1155InvalidSender {
                sender: from,
            }));
        }
        self._update_with_acceptance_check(
            from,
            Address::ZERO,
            ids,
            values,
            vec![].into(),
        )?;
        Ok(())
    }

    // TODO
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is the `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `from` is the `Address::ZERO`, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`TransferSingle`] event if the arrays contain one element, and
    /// [`TransferBatch`] otherwise.
    fn do_safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver: to,
            }));
        }
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC1155InvalidSender {
                sender: from,
            }));
        }
        self._update_with_acceptance_check(from, to, ids, values, data)
    }

    /// Transfers a `value` amount of `token_id` from `from` to
    /// `to`. Will mint (or burn) if `from` (or `to`) is the `Address::ZERO`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id.
    /// * `value` - Amount of tokens to be transferred.
    ///
    /// # Errors
    ///
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`Error::InsufficientBalance`] is returned.
    ///
    ///
    /// # Panics
    ///
    /// If balance exceeds `U256::MAX`. It may happen during `mint` operation.
    fn do_update(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Error> {
        if !from.is_zero() {
            let from_balance = self.balance_of(from, token_id);
            if from_balance < value {
                return Err(Error::InsufficientBalance(
                    ERC1155InsufficientBalance {
                        sender: from,
                        balance: from_balance,
                        needed: value,
                        token_id,
                    },
                ));
            }
            self._balances
                .setter(token_id)
                .setter(from)
                .sub_assign_unchecked(value);
        }

        if !to.is_zero() {
            let new_balance = self
                ._balances
                .setter(token_id)
                .setter(to)
                .checked_add(value)
                .expect("should not exceed `U256::MAX` for `_balances`");
            self._balances.setter(token_id).setter(to).set(new_balance);
        }

        Ok(())
    }

    /// Checks if `ids` array has same length as `values`.
    ///
    /// # Arguments
    ///
    /// * `ids` - array of `ids`.
    /// * `values` - array of `values`.
    ///
    /// # Errors
    ///
    /// If length of `ids` is not equal to length of `values`, then the error
    /// [`Error::InvalidArrayLength`] is returned.
    fn require_equal_arrays<T, U>(
        ids: &[T],
        values: &[U],
    ) -> Result<(), Error> {
        if ids.len() != values.len() {
            return Err(Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length: U256::from(ids.len()),
                values_length: U256::from(values.len()),
            }));
        }
        Ok(())
    }

    /// Checks if `sender` is authorized to transfer tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    ///
    /// # Errors
    ///
    /// If the `from` is not the caller (`msg::sender()`),
    /// and the caller does not have the right to approve, then the error
    /// [`Error::MissingApprovalForAll`] is returned.
    ///
    /// # Requirements
    ///
    /// * If the caller is not `from`, it must have been approved to spend
    ///   `from`'s tokens via [`IErc1155::set_approval_for_all`].
    fn authorize_transfer(&self, from: Address) -> Result<(), Error> {
        let sender = msg::sender();
        if from != sender && !self.is_approved_for_all(from, sender) {
            return Err(Error::MissingApprovalForAll(
                ERC1155MissingApprovalForAll { operator: sender, owner: from },
            ));
        }

        Ok(())
    }
}
#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use super::{
        ERC1155InsufficientBalance, ERC1155InvalidArrayLength,
        ERC1155InvalidOperator, ERC1155InvalidReceiver, ERC1155InvalidSender,
        ERC1155MissingApprovalForAll, Erc1155, Erc1155ReceiverData, Error,
        IErc1155, Transfer, BATCH_TRANSFER_FN_SELECTOR,
        SINGLE_TRANSFER_FN_SELECTOR,
    };
    use crate::{
        token::erc721::IErc721, utils::introspection::erc165::IErc165,
    };

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");
    const CHARLIE: Address =
        address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(U256::from).collect()
    }

    pub(crate) fn random_values(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
    }

    fn init(
        contract: &mut Erc1155,
        receiver: Address,
        size: usize,
    ) -> (Vec<U256>, Vec<U256>) {
        let token_ids = random_token_ids(size);
        let values = random_values(size);

        contract
            ._mint_batch(
                receiver,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("Mint failed");
        (token_ids, values)
    }

    #[test]
    fn should_create_transfer_single() {
        let id = uint!(1_U256);
        let value = uint!(10_U256);
        let details = Erc1155ReceiverData::new(vec![id], vec![value]);
        assert_eq!(SINGLE_TRANSFER_FN_SELECTOR, details.fn_selector);
        assert_eq!(Transfer::Single { id, value }, details.transfer);
    }

    #[test]
    fn should_create_transfer_batch() {
        let ids = random_token_ids(5);
        let values = random_values(5);
        let details = Erc1155ReceiverData::new(ids.clone(), values.clone());
        assert_eq!(BATCH_TRANSFER_FN_SELECTOR, details.fn_selector);
        assert_eq!(Transfer::Batch { ids, values }, details.transfer);
    }

    #[motsu::test]
    fn balance_of_zero_balance(contract: Erc1155) {
        let owner = msg::sender();
        let token_id = random_token_ids(1)[0];
        let balance = contract.balance_of(owner, token_id);
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn error_when_array_length_mismatch(contract: Erc1155) {
        let token_ids = random_token_ids(3);
        let accounts = vec![ALICE, BOB, DAVE, CHARLIE];
        let ids_length = U256::from(token_ids.len());
        let accounts_length = U256::from(accounts.len());

        let err = contract
            .balance_of_batch(accounts, token_ids)
            .expect_err("should return `Error::InvalidArrayLength`");

        assert!(matches!(
            err,
            Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length: ids_l,
                values_length: accounts_l,
            }) if ids_l == ids_length && accounts_l == accounts_length
        ));
    }

    #[motsu::test]
    fn balance_of_batch_zero_balance(contract: Erc1155) {
        let token_ids = random_token_ids(4);
        let accounts = vec![ALICE, BOB, DAVE, CHARLIE];
        let balances = contract
            .balance_of_batch(accounts, token_ids)
            .expect("should return a vector of `U256::ZERO`");

        for balance in balances {
            assert_eq!(U256::ZERO, balance);
        }
    }

    #[motsu::test]
    fn set_approval_for_all(contract: Erc1155) {
        let alice = msg::sender();
        contract._operator_approvals.setter(alice).setter(BOB).set(false);

        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");
        assert_eq!(contract.is_approved_for_all(alice, BOB), true);

        contract.set_approval_for_all(BOB, false).expect(
            "should disapprove Bob for operations on all Alice's tokens",
        );
        assert_eq!(contract.is_approved_for_all(alice, BOB), false);
    }

    #[motsu::test]
    fn error_when_invalid_operator_set_approval_for_all(contract: Erc1155) {
        let invalid_operator = Address::ZERO;

        let err = contract
            .set_approval_for_all(invalid_operator, true)
            .expect_err("should not approve for all for invalid operator");

        assert!(matches!(
            err,
            Error::InvalidOperator(ERC1155InvalidOperator {
                operator
            }) if operator == invalid_operator
        ));
    }

    #[motsu::test]
    fn mints(contract: Erc1155) {
        let alice = msg::sender();
        let token_id = random_token_ids(1)[0];
        let value = random_values(1)[0];

        contract
            ._mint(alice, token_id, value, vec![0, 1, 2, 3].into())
            .expect("should mint tokens for Alice");

        let balance = contract.balance_of(alice, token_id);

        assert_eq!(balance, value);
    }

    #[motsu::test]
    fn mints_batch(contract: Erc1155) {
        let token_ids = random_token_ids(4);
        let values = random_values(4);
        let accounts = vec![ALICE, BOB, DAVE, CHARLIE];

        contract
            ._mint_batch(
                ALICE,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("should mint tokens for Alice");
        token_ids.iter().zip(values.iter()).for_each(|(&token_id, &value)| {
            let balance = contract.balance_of(ALICE, token_id);
            assert_eq!(balance, value);
        });

        contract
            ._mint_batch(
                BOB,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("should mint tokens for BOB");
        token_ids.iter().zip(values.iter()).for_each(|(&token_id, &value)| {
            let balance = contract.balance_of(BOB, token_id);
            assert_eq!(balance, value);
        });

        contract
            ._mint_batch(
                DAVE,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("should mint tokens for DAVE");
        token_ids.iter().zip(values.iter()).for_each(|(&token_id, &value)| {
            let balance = contract.balance_of(DAVE, token_id);
            assert_eq!(balance, value);
        });

        contract
            ._mint_batch(
                CHARLIE,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("should mint tokens for CHARLIE");
        token_ids.iter().zip(values.iter()).for_each(|(&token_id, &value)| {
            let balance = contract.balance_of(CHARLIE, token_id);
            assert_eq!(balance, value);
        });

        let balances = contract
            .balance_of_batch(accounts.clone(), token_ids.clone())
            .expect("should return the balances of all accounts");

        balances.iter().zip(values.iter()).for_each(|(&balance, &value)| {
            assert_eq!(balance, value);
        });
    }

    #[motsu::test]
    fn safe_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 2);
        let amount_one = values[0] - uint!(1_U256);
        let amount_two = values[1] - uint!(1_U256);

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        contract
            .safe_transfer_from(
                BOB,
                DAVE,
                token_ids[0],
                amount_one,
                vec![].into(),
            )
            .expect("should transfer tokens from Alice to Bob");
        contract
            .safe_transfer_from(
                BOB,
                DAVE,
                token_ids[1],
                amount_two,
                vec![].into(),
            )
            .expect("should transfer tokens from Alice to Bob");

        let balance_id_one = contract.balance_of(DAVE, token_ids[0]);
        let balance_id_two = contract.balance_of(DAVE, token_ids[1]);

        assert_eq!(amount_one, balance_id_one);
        assert_eq!(amount_two, balance_id_two);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 1);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .safe_transfer_from(
                alice,
                invalid_receiver,
                token_ids[0],
                values[0],
                vec![].into(),
            )
            .expect_err("should not transfer tokens to the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 1);
        let invalid_sender = Address::ZERO;

        contract
            ._operator_approvals
            .setter(invalid_sender)
            .setter(alice)
            .set(true);

        let err = contract
            .safe_transfer_from(
                invalid_sender,
                BOB,
                token_ids[0],
                values[0],
                vec![].into(),
            )
            .expect_err("should not transfer tokens from the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_transfer_from(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 1);

        let err = contract
            .safe_transfer_from(
                ALICE,
                BOB,
                token_ids[0],
                values[0],
                vec![].into(),
            )
            .expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == msg::sender() && owner == ALICE
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 1);

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let err = contract
            .safe_transfer_from(
                BOB,
                DAVE,
                token_ids[0],
                values[0] + uint!(1_U256),
                vec![].into(),
            )
            .expect_err("should not transfer tokens with insufficient balance");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == BOB && balance == values[0] && needed == values[0] + uint!(1_U256) && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn safe_transfer_from_with_data(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, DAVE, 1);

        contract._operator_approvals.setter(DAVE).setter(alice).set(true);

        contract
            .safe_transfer_from(
                DAVE,
                CHARLIE,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .expect("should transfer tokens from Alice to Bob");

        let balance = contract.balance_of(CHARLIE, token_ids[0]);

        assert_eq!(values[0], balance);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let (token_ids, values) = init(contract, DAVE, 1);
        let invalid_receiver = Address::ZERO;

        let err = contract
            ._safe_transfer_from(
                DAVE,
                invalid_receiver,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens to the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 1);
        let invalid_sender = Address::ZERO;

        contract
            ._operator_approvals
            .setter(invalid_sender)
            .setter(alice)
            .set(true);

        let err = contract
            .safe_transfer_from(
                invalid_sender,
                CHARLIE,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens from the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let (token_ids, values) = init(contract, ALICE, 1);

        let err = contract
            .safe_transfer_from(
                ALICE,
                BOB,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == msg::sender() && owner == ALICE
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 1);

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let err = contract
            .safe_transfer_from(
                BOB,
                DAVE,
                token_ids[0],
                values[0] + uint!(1_U256),
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens with insufficient balance");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == BOB && balance == values[0] && needed == values[0] + uint!(1_U256) && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn safe_batch_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, DAVE, 2);
        let amount_one = values[0] - uint!(1_U256);
        let amount_two = values[1] - uint!(1_U256);

        contract._operator_approvals.setter(DAVE).setter(alice).set(true);

        contract
            .safe_batch_transfer_from(
                DAVE,
                BOB,
                token_ids.clone(),
                vec![amount_one, amount_two],
                vec![].into(),
            )
            .expect("should transfer tokens from Alice to Bob");

        let balance_id_one = contract.balance_of(BOB, token_ids[0]);
        let balance_id_two = contract.balance_of(BOB, token_ids[1]);

        assert_eq!(amount_one, balance_id_one);
        assert_eq!(amount_two, balance_id_two);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_batch_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .safe_batch_transfer_from(
                alice,
                invalid_receiver,
                token_ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .expect_err("should not transfer tokens to the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_batch_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);
        let invalid_sender = Address::ZERO;

        contract
            ._operator_approvals
            .setter(invalid_sender)
            .setter(alice)
            .set(true);

        let err = contract
            .safe_batch_transfer_from(
                invalid_sender,
                CHARLIE,
                token_ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .expect_err("should not transfer tokens from the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_batch_transfer_from(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 2);

        let err = contract
            .safe_batch_transfer_from(
                ALICE,
                BOB,
                token_ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == msg::sender() && owner == ALICE
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_batch_transfer_from(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, CHARLIE, 2);

        contract._operator_approvals.setter(CHARLIE).setter(alice).set(true);

        let err = contract
            .safe_batch_transfer_from(
                CHARLIE,
                BOB,
                token_ids.clone(),
                vec![values[0] + uint!(1_U256), values[1]],
                vec![].into(),
            )
            .expect_err("should not transfer tokens with insufficient balance");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == CHARLIE && balance == values[0] && needed == values[0] + uint!(1_U256) && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn safe_batch_transfer_from_with_data(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, DAVE, 2);

        contract._operator_approvals.setter(DAVE).setter(alice).set(true);

        contract
            .safe_batch_transfer_from(
                DAVE,
                BOB,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("should transfer tokens from Alice to Bob");

        let balance_id_one = contract.balance_of(BOB, token_ids[0]);
        let balance_id_two = contract.balance_of(BOB, token_ids[1]);

        assert_eq!(values[0], balance_id_one);
        assert_eq!(values[1], balance_id_two);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_batch_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .safe_batch_transfer_from(
                alice,
                invalid_receiver,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens to the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_batch_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);
        let invalid_sender = Address::ZERO;

        contract
            ._operator_approvals
            .setter(invalid_sender)
            .setter(alice)
            .set(true);

        let err = contract
            .safe_batch_transfer_from(
                invalid_sender,
                CHARLIE,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens from the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_batch_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let (token_ids, values) = init(contract, ALICE, 2);

        let err = contract
            .safe_batch_transfer_from(
                ALICE,
                BOB,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == msg::sender() && owner == ALICE
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_batch_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, CHARLIE, 2);

        contract._operator_approvals.setter(CHARLIE).setter(alice).set(true);

        let err = contract
            .safe_batch_transfer_from(
                CHARLIE,
                BOB,
                token_ids.clone(),
                vec![values[0] + uint!(1_U256), values[1]],
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer tokens with insufficient balance");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == CHARLIE && balance == values[0] && needed == values[0] + uint!(1_U256) && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc1155 as IErc1155>::INTERFACE_ID;
        let expected = 0xd9b67a26;
        assert_eq!(actual, expected);

        let actual = <Erc1155 as IErc165>::INTERFACE_ID;
        let expected = 0x01ffc9a7;
        assert_eq!(actual, expected);
    }
}
