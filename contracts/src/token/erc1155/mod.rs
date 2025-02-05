//! Implementation of the ERC-1155 token standard.
use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    call::{self, Call, MethodError},
    evm, function_selector, msg,
    prelude::{public, storage, AddressVM, SolidityError},
    storage::{StorageBool, StorageMap, StorageU256, TopLevelStorage},
};

use crate::utils::{
    introspection::erc165::{Erc165, IErc165},
    math::storage::{AddAssignChecked, SubAssignUnchecked},
};

pub mod extensions;
mod receiver;
pub use receiver::IERC1155Receiver;

/// The expected value returned from [`IERC1155Receiver::on_erc_1155_received`].
pub const SINGLE_TRANSFER_FN_SELECTOR: [u8; 4] = function_selector!(
    "onERC1155Received",
    Address,
    Address,
    U256,
    U256,
    Bytes
);

/// The expected value returned from
/// [`IERC1155Receiver::on_erc_1155_batch_received`].
pub const BATCH_TRANSFER_FN_SELECTOR: [u8; 4] = function_selector!(
    "onERC1155BatchReceived",
    Address,
    Address,
    Vec<U256>,
    Vec<U256>,
    Bytes
);

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

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
    ///
    /// Since encoding [`stylus_sdk::call::Error`] returns the underlying
    /// return data, this error will be encoded either as `Error(string)` or
    /// `Panic(uint256)`, as those are the built-in errors emitted by default
    /// by Solidity's special functions `assert`, `require`, and `revert`.
    ///
    /// See: <https://docs.soliditylang.org/en/v0.8.28/control-structures.html#error-handling-assert-require-revert-and-exceptions>
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

/// State of an [`Erc1155`] token.
#[storage]
pub struct Erc1155 {
    /// Maps users to balances.
    #[allow(clippy::used_underscore_binding)]
    pub _balances: StorageMap<U256, StorageMap<Address, StorageU256>>,
    /// Maps owners to a mapping of operator approvals.
    #[allow(clippy::used_underscore_binding)]
    pub _operator_approvals:
        StorageMap<Address, StorageMap<Address, StorageBool>>,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc1155 {}

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
    /// * `account` - Account of the token's owner.
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
    /// # Errors
    ///
    /// * [`Error::InvalidArrayLength`] -  If the length of `accounts` is not
    ///   equal to the length of `ids`.
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
    /// * [`Error::InvalidOperator`] - If `operator` is `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`ApprovalForAll`].
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
    /// * [`Error::InvalidReceiver`] - Returned when `to` is `Address::ZERO` or
    ///   when [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidSender`] - Returned when `from` is `Address::ZERO`.
    /// * [`Error::MissingApprovalForAll`] - Returned when `from` is not the
    ///   caller (`msg::sender()`), and the caller does not have the right to
    ///   approve.
    /// * [`Error::InsufficientBalance`] - Returned when `value` is greater than
    ///   the balance of the `from` account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`].

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
    /// * [`Error::InvalidReceiver`] - Returned when `to` is `Address::ZERO` or
    ///   when [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned
    ///   its interface id or returned with error.
    /// * [`Error::InvalidSender`] - Returned when `from` is `Address::ZERO`.
    /// * [`Error::InvalidArrayLength`] - Returned when the length of `ids` is
    ///   not equal to the length of `values`.
    /// * [`Error::InsufficientBalance`] - Returned when any of the `values` is
    ///   greater than the balance of the `from` account.
    /// * [`Error::MissingApprovalForAll`] - Returned when `from` is not the
    ///   caller (`msg::sender()`), and the caller does not have the right to
    ///   approve.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.

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
        Self::require_equal_arrays_length(&ids, &accounts)?;

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
        self.do_safe_transfer_from(from, to, vec![id], vec![value], &data)
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
        self.do_safe_transfer_from(from, to, ids, values, &data)
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
    /// NOTE: The ERC-1155 acceptance check is not performed in this function.
    /// See [`Self::_update_with_acceptance_check`] instead.
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
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`Error::InsufficientBalance`] - If `value` is greater than the
    ///   balance of the `from` account.
    ///
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`, may happen during `mint`
    /// operation.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        Self::require_equal_arrays_length(&ids, &values)?;

        let operator = msg::sender();

        for (&token_id, &value) in ids.iter().zip(values.iter()) {
            self.do_update(from, to, token_id, value)?;
        }

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
    /// [`IERC1155Receiver::on_erc_1155_batch_received`] on the receiver address
    /// if it contains code.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidArrayLength`] - Returned when length of `ids` is not
    ///   equal to length of `values`.
    /// * [`Error::InsufficientBalance`] - Returned when `value` is greater than
    ///   the balance of the `from` account.
    /// * [`Error::InvalidReceiver`] - Returned when
    ///   [`IERC1155Receiver::on_erc_1155_received`] or
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`, may happen during `mint`
    ///   operation.
    fn _update_with_acceptance_check(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
    ) -> Result<(), Error> {
        self._update(from, to, ids.clone(), values.clone())?;

        if !to.is_zero() {
            self._check_on_erc1155_received(
                msg::sender(),
                from,
                to,
                Erc1155ReceiverData::new(ids, values),
                data.to_vec().into(),
            )?;
        }

        Ok(())
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
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`].
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`.
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        value: U256,
        data: &Bytes,
    ) -> Result<(), Error> {
        self._do_mint(to, vec![id], vec![value], data)
    }

    /// Batched version of [`Self::_mint`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all tokens ids to be minted.
    /// * `values` - Array of all amounts of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] -  If `to` is `Address::ZERO`.
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`IERC1155Receiver::on_erc_1155_received`] - If  hasn't returned its
    /// * [`Error::InvalidReceiver`] - interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`.
    pub fn _mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
    ) -> Result<(), Error> {
        self._do_mint(to, ids, values, data)
    }

    /// Destroys a `value` amount of tokens of type `id` from `from`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `id` - Token id to be burnt.
    /// * `value` - Amount of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is the `Address::ZERO`.
    /// * [`Error::InsufficientBalance`]  - If `value` is greater than the
    ///   balance of the `from` account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`].

    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        value: U256,
    ) -> Result<(), Error> {
        self._do_burn(from, vec![id], vec![value])
    }

    /// Batched version of [`Self::_burn`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `ids` - Array of all tokens ids to be burnt.
    /// * `values` - Array of all amounts of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is the `Address::ZERO`.
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`Error::InsufficientBalance`] - If any of the `values` is greater
    ///   than the balance of the respective token from `tokens` of the `from`
    ///   account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.

    pub fn _burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        self._do_burn(from, ids, values)
    }

    /// Approve `operator` to operate on all of `owner` tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Tokens owner (`msg::sender`).
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `approved` - Flag that determines whether or not permission will be
    ///   granted to `operator`. If true, this means `operator` will be allowed
    ///   to manage `owner`'s assets.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidOperator`] - If `operator` is the `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`ApprovalForAll`].
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
    /// Performs an acceptance check for the provided `operator` by calling
    /// [`IERC1155Receiver::on_erc_1155_received`] in case of single token
    /// transfer, or [`IERC1155Receiver::on_erc_1155_batch_received`] in
    /// case of batch transfer on the `to` address.
    ///
    /// The acceptance call is not executed and treated as a no-op if the
    /// target address doesn't contain code (i.e. an EOA). Otherwise,
    /// the recipient must implement either
    /// [`IERC1155Receiver::on_erc_1155_received`] for single transfer, or
    /// [`IERC1155Receiver::on_erc_1155_batch_received`] for a batch transfer,
    /// and return the acceptance value to accept the transfer.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Generally the address that initiated the token transfer
    ///   (e.g. `msg::sender()`).
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `details` - Details about token transfer, check
    ///   [`Erc1155ReceiverData`].
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    fn _check_on_erc1155_received(
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
                    if !reason.is_empty() {
                        // Non-IERC1155Receiver implementer.
                        return Err(e.into());
                    }
                }

                return Err(ERC1155InvalidReceiver { receiver: to }.into());
            }
        };

        // Token rejected.
        if id != details.receiver_fn_selector {
            return Err(ERC1155InvalidReceiver { receiver: to }.into());
        }

        Ok(())
    }

    /// Creates `values` of tokens specified by `ids`, and assigns
    /// them to `to`. Performs the token acceptance check by
    /// calling [`IERC1155Receiver::on_erc_1155_received`] or
    /// [`IERC1155Receiver::on_erc_1155_batch_received`] on the `to` address if
    /// it contains code.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids to be minted.
    /// * `values` - Array of all amounts of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidArrayLength`] -  If length of `ids` is not equal to
    ///   length of `values`.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the array contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`.
    fn _do_mint(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
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

    /// Destroys `values` amounts of tokens specified by `ids` from `from`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `ids` - Array of all token ids to be burnt.
    /// * `values` - Array of all amount of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `from` is the `Address::ZERO`.
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`Error::InsufficientBalance`] -If any of the `values` is greater than
    ///   the balance of the respective token from `tokens` of the `from`
    ///   account.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.

    fn _do_burn(
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
            &vec![].into(),
        )?;
        Ok(())
    }

    /// Transfers `values` of tokens specified by `ids` from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is the `Address::ZERO`.
    /// * [`Error::InvalidSender`] - If `from` is the `Address::ZERO`.
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`Error::InsufficientBalance`] - If `value` is greater than the
    ///   balance of the `from` account.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`.
    fn do_safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
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
    /// * [`Error::InsufficientBalance`] - If `value` is greater than the
    ///   balance of the `from` account.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds `U256::MAX`.
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
            self._balances.setter(token_id).setter(to).add_assign_checked(
                value,
                "should not exceed `U256::MAX` for `_balances`",
            );
        }

        Ok(())
    }

    /// Checks if `ids` array has same length as `values` array.
    ///
    /// # Arguments
    ///
    /// * `ids` - array of `ids`.
    /// * `values` - array of `values`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    fn require_equal_arrays_length<T, U>(
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

    /// Checks if `msg::sender()` is authorized to transfer tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    ///
    /// # Errors
    ///
    /// * [`Error::MissingApprovalForAll`] -  If the `from` is not the caller
    ///   (`msg::sender()`), and the caller does not have the right to approve.
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

/// Data struct to be passed to a contract that
/// implements [`IERC1155Receiver`] interface.
struct Erc1155ReceiverData {
    /// ERC-1155 Receiver function selector.
    receiver_fn_selector: [u8; 4],
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
    /// Check [`IERC1155Receiver::on_erc_1155_received`].
    ///
    /// # Arguments
    ///
    /// * `id` - Token id being transferred.
    /// * `value` - Amount of tokens being transferred.
    fn single(id: U256, value: U256) -> Self {
        Self {
            receiver_fn_selector: SINGLE_TRANSFER_FN_SELECTOR,
            transfer: Transfer::Single { id, value },
        }
    }

    /// Creates a new instance for a [`Transfer::Batch`].
    /// Check [`IERC1155Receiver::on_erc_1155_batch_received`].
    ///
    /// # Arguments
    ///
    /// * `ids` - Array of tokens ids being transferred.
    /// * `values` - Array of all amount of tokens being transferred.
    fn batch(ids: Vec<U256>, values: Vec<U256>) -> Self {
        Self {
            receiver_fn_selector: BATCH_TRANSFER_FN_SELECTOR,
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
    use crate::utils::introspection::erc165::IErc165;

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");
    const CHARLIE: Address =
        address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(U256::from).collect()
    }

    pub(crate) fn random_values(size: usize) -> Vec<U256> {
        (1..=size).map(U256::from).collect()
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
                &vec![0, 1, 2, 3].into(),
            )
            .expect("Mint failed");
        (token_ids, values)
    }

    fn append(values: Vec<U256>, value: u64) -> Vec<U256> {
        values.into_iter().chain(std::iter::once(U256::from(value))).collect()
    }

    #[test]
    fn should_create_transfer_single() {
        let id = uint!(1_U256);
        let value = uint!(10_U256);
        let details = Erc1155ReceiverData::new(vec![id], vec![value]);
        assert_eq!(SINGLE_TRANSFER_FN_SELECTOR, details.receiver_fn_selector);
        assert_eq!(Transfer::Single { id, value }, details.transfer);
    }

    #[test]
    fn should_create_transfer_batch() {
        let ids = random_token_ids(5);
        let values = random_values(5);
        let details = Erc1155ReceiverData::new(ids.clone(), values.clone());
        assert_eq!(BATCH_TRANSFER_FN_SELECTOR, details.receiver_fn_selector);
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

        let expected = vec![U256::ZERO; 4];
        assert_eq!(expected, balances);
    }

    #[motsu::test]
    fn set_approval_for_all(contract: Erc1155) {
        let alice = msg::sender();
        contract._operator_approvals.setter(alice).setter(BOB).set(false);

        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");
        assert!(contract.is_approved_for_all(alice, BOB));

        contract.set_approval_for_all(BOB, false).expect(
            "should disapprove Bob for operations on all Alice's tokens",
        );
        assert!(!contract.is_approved_for_all(alice, BOB));
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
            ._mint(alice, token_id, value, &vec![0, 1, 2, 3].into())
            .expect("should mint tokens for Alice");

        let balance = contract.balance_of(alice, token_id);

        assert_eq!(balance, value);
    }

    #[motsu::test]
    fn error_when_mints_to_invalid_receiver(contract: Erc1155) {
        let invalid_receiver = Address::ZERO;
        let token_id = random_token_ids(1)[0];
        let value = random_values(1)[0];

        let err = contract
            ._mint(invalid_receiver, token_id, value, &vec![0, 1, 2, 3].into())
            .expect_err("should not mint tokens for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn mints_batch(contract: Erc1155) {
        let token_ids = random_token_ids(4);
        let values = random_values(4);

        contract
            ._mint_batch(
                ALICE,
                token_ids.clone(),
                values.clone(),
                &vec![0, 1, 2, 3].into(),
            )
            .expect("should batch mint tokens");

        token_ids.iter().zip(values.iter()).for_each(|(&token_id, &value)| {
            assert_eq!(value, contract.balance_of(ALICE, token_id));
        });

        let balances = contract
            .balance_of_batch(vec![ALICE; 4], token_ids.clone())
            .expect("should return balances");

        assert_eq!(values, balances);
    }

    #[motsu::test]
    fn mints_batch_same_token(contract: Erc1155) {
        let token_id = uint!(1_U256);
        let values = random_values(4);
        let expected_balance: U256 = values.iter().sum();

        contract
            ._mint_batch(
                ALICE,
                vec![token_id; 4],
                values.clone(),
                &vec![0, 1, 2, 3].into(),
            )
            .expect("should batch mint tokens");

        assert_eq!(expected_balance, contract.balance_of(ALICE, token_id));

        let balances = contract
            .balance_of_batch(vec![ALICE; 4], vec![token_id; 4])
            .expect("should return balances");

        assert_eq!(vec![expected_balance; 4], balances);
    }

    #[motsu::test]
    fn error_when_batch_mints_to_invalid_receiver(contract: Erc1155) {
        let token_ids = random_token_ids(1);
        let values = random_values(1);
        let invalid_receiver = Address::ZERO;

        let err = contract
            ._mint_batch(
                invalid_receiver,
                token_ids,
                values,
                &vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not batch mint tokens for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_batch_mints_not_equal_arrays(contract: Erc1155) {
        let token_ids = random_token_ids(3);
        let values = random_values(4);

        let err = contract
            ._mint_batch(ALICE, token_ids, values, &vec![0, 1, 2, 3].into())
            .expect_err(
                "should not batch mint tokens when not equal array lengths",
            );

        assert!(matches!(
            err,
            Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length, values_length
            }) if ids_length == uint!(3_U256) && values_length == uint!(4_U256)
        ));
    }

    #[motsu::test]
    fn burns(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 1);
        let token_id = token_ids[0];
        let value = values[0];

        contract._burn(ALICE, token_id, value).expect("should burn tokens");

        let balances = contract.balance_of(ALICE, token_id);

        assert_eq!(U256::ZERO, balances);
    }

    #[motsu::test]
    fn error_when_burns_from_invalid_sender(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 1);
        let invalid_sender = Address::ZERO;

        let err = contract
            ._burn(invalid_sender, token_ids[0], values[0])
            .expect_err("should not burn token for invalid sender");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_burns_with_insufficient_balance(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 1);

        let err = contract
            ._burn(ALICE, token_ids[0], values[0] + uint!(1_U256))
            .expect_err("should not burn token when insufficient balance");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == ALICE && balance == values[0] && needed == values[0] + uint!(1_U256) && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn burns_batch(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 4);

        contract
            ._burn_batch(ALICE, token_ids.clone(), values.clone())
            .expect("should batch burn tokens");

        let balances = contract
            .balance_of_batch(vec![ALICE; 4], token_ids.clone())
            .expect("should return balances");

        assert_eq!(vec![U256::ZERO; 4], balances);
    }

    #[motsu::test]
    fn burns_batch_same_token(contract: Erc1155) {
        let token_id = uint!(1_U256);
        let value = uint!(80_U256);

        contract
            ._mint(ALICE, token_id, value, &vec![0, 1, 2, 3].into())
            .expect("should mint token");

        contract
            ._burn_batch(
                ALICE,
                vec![token_id; 4],
                vec![
                    uint!(20_U256),
                    uint!(10_U256),
                    uint!(30_U256),
                    uint!(20_U256),
                ],
            )
            .expect("should batch burn tokens");

        assert_eq!(U256::ZERO, contract.balance_of(ALICE, token_id));
    }

    #[motsu::test]
    fn error_when_batch_burns_from_invalid_sender(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 4);
        let invalid_sender = Address::ZERO;

        let err = contract
            ._burn_batch(invalid_sender, token_ids, values)
            .expect_err("should not batch burn tokens for invalid sender");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_batch_burns_with_insufficient_balance(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 4);

        let err = contract
            ._burn_batch(
                ALICE,
                token_ids.clone(),
                values.clone().into_iter().map(|x| x + uint!(1_U256)).collect(),
            )
            .expect_err(
                "should not batch burn tokens when insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == ALICE && balance == values[0] && needed == values[0] + uint!(1_U256) && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn error_when_batch_burns_not_equal_arrays(contract: Erc1155) {
        let (token_ids, values) = init(contract, ALICE, 3);

        let err = contract
            ._burn_batch(ALICE, token_ids, append(values, 4))
            .expect_err(
                "should not batch burn tokens when not equal array lengths",
            );

        assert!(matches!(
            err,
            Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length, values_length
            }) if ids_length == uint!(3_U256) && values_length == uint!(4_U256)
        ));
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
            .do_safe_transfer_from(
                DAVE,
                invalid_receiver,
                token_ids,
                values,
                &vec![0, 1, 2, 3].into(),
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
    fn error_when_not_equal_arrays_safe_batch_transfer_from(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);

        contract._operator_approvals.setter(DAVE).setter(alice).set(true);

        let err = contract
            .safe_batch_transfer_from(
                DAVE,
                CHARLIE,
                token_ids.clone(),
                append(values, 4),
                vec![].into(),
            )
            .expect_err(
                "should not transfer tokens when not equal array lengths",
            );

        assert!(matches!(
            err,
            Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length, values_length
            }) if ids_length == uint!(4_U256) && values_length == uint!(5_U256)
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
    fn error_when_not_equal_arrays_safe_batch_transfer_from_with_data(
        contract: Erc1155,
    ) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);

        contract._operator_approvals.setter(DAVE).setter(alice).set(true);

        let err = contract
            .safe_batch_transfer_from(
                DAVE,
                CHARLIE,
                token_ids.clone(),
                append(values, 4),
                vec![0, 1, 2, 3].into(),
            )
            .expect_err(
                "should not transfer tokens when not equal array lengths",
            );

        assert!(matches!(
            err,
            Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length, values_length
            }) if ids_length == uint!(4_U256) && values_length == uint!(5_U256)
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
