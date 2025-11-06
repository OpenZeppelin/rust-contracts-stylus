//! Implementation of the ERC-1155 token standard.
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    call::{self, Call, MethodError},
    evm, msg,
    prelude::*,
    storage::{StorageBool, StorageMap, StorageU256},
};

use crate::utils::{
    introspection::erc165::IErc165,
    math::storage::{AddAssignChecked, SubAssignUnchecked},
};

pub mod abi;
pub mod extensions;
pub mod receiver;
pub mod utils;

pub use abi::Erc1155ReceiverInterface;
pub use receiver::{
    IErc1155Receiver, BATCH_TRANSFER_FN_SELECTOR, SINGLE_TRANSFER_FN_SELECTOR,
};
pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when `value` amount of tokens of type `id` are
        /// transferred from `from` to `to` by `operator`.
        #[derive(Debug)]
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
        #[derive(Debug)]
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
        #[derive(Debug)]
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

        /// Indicates a failure with the receiver reverting with a reason.
        ///
        /// * `reason` - Revert reason.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error InvalidReceiverWithReason(string reason);
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
    InvalidReceiverWithReason(InvalidReceiverWithReason),
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

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc1155`] token.
#[storage]
pub struct Erc1155 {
    /// Maps users to balances.
    pub(crate) balances: StorageMap<U256, StorageMap<Address, StorageU256>>,
    /// Maps owners to a mapping of operator approvals.
    pub(crate) operator_approvals:
        StorageMap<Address, StorageMap<Address, StorageBool>>,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc1155 {}

/// Required interface of an [`Erc1155`] compliant contract.
#[interface_id]
pub trait IErc1155: IErc165 {
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
    /// * [`Error::InvalidOperator`] - If `operator` is [`Address::ZERO`].
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
    /// * [`Error::InvalidReceiver`] - Returned when `to` is [`Address::ZERO`]
    ///   or when [`IErc1155Receiver::on_erc1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidSender`] - Returned when `from` is [`Address::ZERO`].
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
    /// * [`Error::InvalidReceiver`] - Returned when `to` is [`Address::ZERO`]
    ///   or when [`IErc1155Receiver::on_erc1155_batch_received`] hasn't
    ///   returned its interface id or returned with error.
    /// * [`Error::InvalidSender`] - Returned when `from` is [`Address::ZERO`].
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
#[implements(IErc1155<Error = Error>, IErc165)]
impl Erc1155 {}

#[public]
impl IErc1155 for Erc1155 {
    type Error = Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.balances.get(id).get(account)
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
        self.operator_approvals.get(account).get(operator)
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

#[public]
impl IErc165 for Erc1155 {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc1155>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

impl Erc1155 {
    /// Transfers a `value` amount of tokens of type `ids` from `from` to
    /// `to`. Will mint (or burn) if `from` (or `to`) is the [`Address::ZERO`].
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
    /// * If updated balance exceeds [`U256::MAX`], may happen during `mint`
    ///   operation.
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
    /// calling [`IErc1155Receiver::on_erc1155_received`] or
    /// [`IErc1155Receiver::on_erc1155_batch_received`] on the receiver address
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
    ///   [`IErc1155Receiver::on_erc1155_received`] or
    ///   [`IErc1155Receiver::on_erc1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`], may happen during `mint`
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc1155Receiver::on_erc1155_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`].
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
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
    /// * [`Error::InvalidReceiver`] -  If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`IErc1155Receiver::on_erc1155_received`] - If  hasn't returned its
    /// * [`Error::InvalidReceiver`] - interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc1155Receiver::on_erc1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
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
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
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
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
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
    /// * [`Error::InvalidOperator`] - If `operator` is the [`Address::ZERO`].
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
        self.operator_approvals.setter(owner).setter(operator).set(approved);
        evm::log(ApprovalForAll { account: owner, operator, approved });
        Ok(())
    }
}

impl Erc1155 {
    /// Performs an acceptance check for the provided `operator` by calling
    /// [`IErc1155Receiver::on_erc1155_received`] in case of single token
    /// transfer, or [`IErc1155Receiver::on_erc1155_batch_received`] in
    /// case of batch transfer on the `to` address.
    ///
    /// The acceptance call is not executed and treated as a no-op if the
    /// target address doesn't contain code (i.e. an EOA). Otherwise,
    /// the recipient must implement either
    /// [`IErc1155Receiver::on_erc1155_received`] for single transfer, or
    /// [`IErc1155Receiver::on_erc1155_batch_received`] for a batch transfer,
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
    ///   [`IErc1155Receiver::on_erc1155_received`] or
    ///   [`IErc1155Receiver::on_erc1155_batch_received`] haven't returned the
    ///   interface id or returned an error.
    /// * [`Error::InvalidReceiverWithReason`] - If
    ///   [`IErc1155Receiver::on_erc1155_received`] or
    ///   [`IErc1155Receiver::on_erc1155_batch_received`] reverted with revert
    ///   data.
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

        let receiver = Erc1155ReceiverInterface::new(to);
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
                        return Err(Error::InvalidReceiverWithReason(
                            InvalidReceiverWithReason {
                                reason: String::from_utf8_lossy(reason)
                                    .to_string(),
                            },
                        ));
                    }
                }

                // Non [`IErc1155Receiver`] implementer.
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
    /// calling [`IErc1155Receiver::on_erc1155_received`] or
    /// [`IErc1155Receiver::on_erc1155_batch_received`] on the `to` address if
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc1155Receiver::on_erc1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc1155Receiver::on_erc1155_batch_received`] hasn't returned its
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
    /// * If updated balance exceeds [`U256::MAX`].
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
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
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
    /// * [`Error::InvalidReceiver`] - If `to` is the [`Address::ZERO`].
    /// * [`Error::InvalidSender`] - If `from` is the [`Address::ZERO`].
    /// * [`Error::InvalidArrayLength`] - If length of `ids` is not equal to
    ///   length of `values`.
    /// * [`Error::InsufficientBalance`] - If `value` is greater than the
    ///   balance of the `from` account.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc1155Receiver::on_erc1155_received`] hasn't returned its
    ///   interface id or returned with error.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc1155Receiver::on_erc1155_batch_received`] hasn't returned its
    ///   interface id or returned with error.
    ///
    /// # Events
    ///
    /// * [`TransferSingle`] - If the arrays contain one element.
    /// * [`TransferBatch`] - If the arrays contain multiple elements.
    ///
    /// # Panics
    ///
    /// * If updated balance exceeds [`U256::MAX`].
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
    /// `to`. Will mint (or burn) if `from` (or `to`) is the [`Address::ZERO`].
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
    /// * If updated balance exceeds [`U256::MAX`].
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
            self.balances
                .setter(token_id)
                .setter(from)
                .sub_assign_unchecked(value);
        }

        if !to.is_zero() {
            self.balances.setter(token_id).setter(to).add_assign_checked(
                value,
                "should not exceed `U256::MAX` for `balances`",
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
/// implements [`IErc1155Receiver`] interface.
struct Erc1155ReceiverData {
    /// ERC-1155 Receiver function selector.
    receiver_fn_selector: B32,
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
    /// Check [`IErc1155Receiver::on_erc1155_received`].
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
    /// Check [`IErc1155Receiver::on_erc1155_batch_received`].
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{aliases::B32, uint, Address, U256};
    use motsu::prelude::*;

    use super::*;
    use crate::{
        token::erc1155::receiver::tests::{
            BadSelectorReceiver1155, EmptyReasonReceiver1155,
            MisdeclaredReceiver1155, RevertingReceiver1155,
            SuccessReceiver1155,
        },
        utils::introspection::erc165::IErc165,
    };

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(U256::from).collect()
    }

    pub(crate) fn random_values(size: usize) -> Vec<U256> {
        (1..=size).map(U256::from).collect()
    }

    trait Init {
        fn init(
            &mut self,
            receiver: Address,
            size: usize,
        ) -> (Vec<U256>, Vec<U256>);
    }

    impl Init for Erc1155 {
        fn init(
            &mut self,
            receiver: Address,
            size: usize,
        ) -> (Vec<U256>, Vec<U256>) {
            let token_ids = random_token_ids(size);
            let values = random_values(size);

            self._mint_batch(
                receiver,
                token_ids.clone(),
                values.clone(),
                &vec![0, 1, 2, 3].into(),
            )
            .expect("Mint failed");
            (token_ids, values)
        }
    }

    fn append(values: Vec<U256>, value: u64) -> Vec<U256> {
        values.into_iter().chain(std::iter::once(U256::from(value))).collect()
    }

    #[test]
    fn should_create_transfer_single() {
        let id = U256::ONE;
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
    fn balance_of_zero_balance(contract: Contract<Erc1155>, alice: Address) {
        let owner = alice;
        let token_id = random_token_ids(1)[0];
        let balance = contract.sender(alice).balance_of(owner, token_id);
        assert_eq!(U256::ZERO, balance);
    }

    // -------------------- _check_on_erc1155_received -----------------------

    #[motsu::test]
    fn check_on_received_rejects_wrong_selector_on_mint(
        contract: Contract<Erc1155>,
        bad_receiver: Contract<BadSelectorReceiver1155>,
        alice: Address,
    ) {
        let id = U256::ONE;
        let value = uint!(5_U256);

        let err = contract
            .sender(alice)
            ._mint(bad_receiver.address(), id, value, &vec![].into())
            .motsu_expect_err(
                "receiver returning wrong selector must be rejected",
            );

        assert!(
            matches!(err, Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == bad_receiver.address())
        );

        // State unchanged for receiver
        assert_eq!(
            U256::ZERO,
            contract.sender(alice).balance_of(bad_receiver.address(), id)
        );
    }

    #[motsu::test]
    fn check_on_received_bubbles_revert_reason_on_mint(
        contract: Contract<Erc1155>,
        reverting_receiver: Contract<RevertingReceiver1155>,
        alice: Address,
    ) {
        let id = uint!(2_U256);
        let value = uint!(7_U256);

        let err = contract
            .sender(alice)
            ._mint(reverting_receiver.address(), id, value, &vec![].into())
            .motsu_expect_err(
                "receiver reverting should return InvalidReceiverWithReason",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiverWithReason(InvalidReceiverWithReason { reason })
                if reason == "Receiver rejected single"
        ));

        assert_eq!(
            U256::ZERO,
            contract.sender(alice).balance_of(reverting_receiver.address(), id)
        );
    }

    // --------------- _update_with_acceptance_check error path --------------

    #[motsu::test]
    fn update_with_acceptance_check_reverts_state_on_rejection(
        contract: Contract<Erc1155>,
        bad_receiver: Contract<BadSelectorReceiver1155>,
        alice: Address,
    ) {
        let id = uint!(3_U256);
        let value = uint!(11_U256);

        // Mint to Alice (EOA)
        contract
            .sender(alice)
            ._mint(alice, id, value, &vec![].into())
            .motsu_expect("mint to EOA should succeed");

        let alice_before = contract.sender(alice).balance_of(alice, id);
        assert_eq!(alice_before, value);

        // Attempt transfer to rejecting contract
        let err = contract
            .sender(alice)
            .safe_transfer_from(
                alice,
                bad_receiver.address(),
                id,
                value,
                vec![].into(),
            )
            .motsu_expect_err("transfer to rejecting receiver must fail");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver { receiver })
                if receiver == bad_receiver.address()
        ));

        // Balances unchanged after failed transfer
        let alice_after = contract.sender(alice).balance_of(alice, id);
        let bad_after =
            contract.sender(alice).balance_of(bad_receiver.address(), id);
        assert_eq!(alice_after, value);
        assert_eq!(bad_after, U256::ZERO);
    }

    // --------------------- Additional coverage tests ----------------------

    // Success flow for single and batch
    #[motsu::test]
    fn check_on_received_success_single_and_batch(
        contract: Contract<Erc1155>,
        receiver: Contract<SuccessReceiver1155>,
        alice: Address,
    ) {
        // single
        let id = uint!(10_U256);
        let value = uint!(3_U256);
        contract
            .sender(alice)
            ._mint(receiver.address(), id, value, &vec![].into())
            .motsu_expect("mint to accepting receiver should succeed");
        assert_eq!(
            value,
            contract.sender(alice).balance_of(receiver.address(), id)
        );

        // batch
        let ids = vec![uint!(21_U256), uint!(22_U256)];
        let vals = vec![uint!(5_U256), uint!(7_U256)];
        contract
            .sender(alice)
            ._mint_batch(
                receiver.address(),
                ids.clone(),
                vals.clone(),
                &vec![].into(),
            )
            .motsu_expect("batch mint to accepting receiver should succeed");
        for (tid, val) in ids.into_iter().zip(vals.into_iter()) {
            assert_eq!(
                val,
                contract.sender(alice).balance_of(receiver.address(), tid)
            );
        }
    }

    // Err(Revert) but empty reason -> InvalidReceiver
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: un-ignore when https://github.com/OpenZeppelin/stylus-test-helpers/issues/118 is fixed"]
    fn check_on_received_empty_reason_revert(
        contract: Contract<Erc1155>,
        empty_reason_receiver: Contract<EmptyReasonReceiver1155>,
        alice: Address,
    ) {
        let id = uint!(100_U256);
        let value = U256::ONE;
        let err = contract
            .sender(alice)
            ._mint(empty_reason_receiver.address(), id, value, &vec![].into())
            .motsu_expect_err("empty revert should map to InvalidReceiver");

        assert!(
            matches!(err, Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == empty_reason_receiver.address())
        );
        assert_eq!(
            U256::ZERO,
            contract
                .sender(alice)
                .balance_of(empty_reason_receiver.address(), id)
        );
    }

    // Err but not Revert (decode error due to empty success return) ->
    // InvalidReceiver
    #[motsu::test]
    fn check_on_received_non_revert_error(
        contract: Contract<Erc1155>,
        misdeclared_receiver: Contract<MisdeclaredReceiver1155>,
        alice: Address,
    ) {
        let id = uint!(200_U256);
        let value = uint!(2_U256);
        let err = contract
            .sender(alice)
            ._mint(misdeclared_receiver.address(), id, value, &vec![].into())
            .motsu_expect_err("decode error should map to InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == misdeclared_receiver.address()
        ));
        assert_eq!(
            U256::ZERO,
            contract
                .sender(alice)
                .balance_of(misdeclared_receiver.address(), id)
        );
    }

    #[motsu::test]
    fn error_when_array_length_mismatch(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
        charlie: Address,
    ) {
        let token_ids = random_token_ids(3);
        let accounts = vec![alice, bob, dave, charlie];
        let ids_length = U256::from(token_ids.len());
        let accounts_length = U256::from(accounts.len());

        let err = contract
            .sender(alice)
            .balance_of_batch(accounts, token_ids)
            .motsu_expect_err("should return `Error::InvalidArrayLength`");

        assert!(matches!(
            err,
            Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length: ids_l,
                values_length: accounts_l,
            }) if ids_l == ids_length && accounts_l == accounts_length
        ));
    }

    #[motsu::test]
    fn balance_of_batch_zero_balance(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
        charlie: Address,
    ) {
        let token_ids = random_token_ids(4);
        let accounts = vec![alice, bob, dave, charlie];
        let balances = contract
            .sender(alice)
            .balance_of_batch(accounts, token_ids)
            .motsu_expect("should return a vector of `U256::ZERO`");

        let expected = vec![U256::ZERO; 4];
        assert_eq!(expected, balances);
    }

    #[motsu::test]
    fn set_approval_for_all(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .operator_approvals
            .setter(alice)
            .setter(bob)
            .set(false);

        contract.sender(alice).set_approval_for_all(bob, true).motsu_expect(
            "should approve Bob for operations on all Alice's tokens",
        );
        assert!(contract.sender(alice).is_approved_for_all(alice, bob));

        contract.sender(alice).set_approval_for_all(bob, false).motsu_expect(
            "should disapprove Bob for operations on all Alice's tokens",
        );
        assert!(!contract.sender(alice).is_approved_for_all(alice, bob));
    }

    #[motsu::test]
    fn error_when_invalid_operator_set_approval_for_all(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let invalid_operator = Address::ZERO;

        let err = contract
            .sender(alice)
            .set_approval_for_all(invalid_operator, true)
            .motsu_expect_err(
                "should not approve for all for invalid operator",
            );

        assert!(matches!(
            err,
            Error::InvalidOperator(ERC1155InvalidOperator {
                operator
            }) if operator == invalid_operator
        ));
    }

    #[motsu::test]
    fn mints(contract: Contract<Erc1155>, alice: Address) {
        let token_id = random_token_ids(1)[0];
        let value = random_values(1)[0];

        contract
            .sender(alice)
            ._mint(alice, token_id, value, &vec![0, 1, 2, 3].into())
            .motsu_expect("should mint tokens for Alice");

        let balance = contract.sender(alice).balance_of(alice, token_id);

        assert_eq!(balance, value);
    }

    #[motsu::test]
    fn error_when_mints_to_invalid_receiver(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;
        let token_id = random_token_ids(1)[0];
        let value = random_values(1)[0];

        let err = contract
            .sender(alice)
            ._mint(invalid_receiver, token_id, value, &vec![0, 1, 2, 3].into())
            .motsu_expect_err("should not mint tokens for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn mints_batch(contract: Contract<Erc1155>, alice: Address) {
        let token_ids = random_token_ids(4);
        let values = random_values(4);

        contract
            .sender(alice)
            ._mint_batch(
                alice,
                token_ids.clone(),
                values.clone(),
                &vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should batch mint tokens");

        token_ids.iter().zip(values.iter()).for_each(|(&token_id, &value)| {
            assert_eq!(
                value,
                contract.sender(alice).balance_of(alice, token_id)
            );
        });

        let balances = contract
            .sender(alice)
            .balance_of_batch(vec![alice; 4], token_ids.clone())
            .motsu_expect("should return balances");

        assert_eq!(values, balances);
    }

    #[motsu::test]
    fn mints_batch_same_token(contract: Contract<Erc1155>, alice: Address) {
        let token_id = U256::ONE;
        let values = random_values(4);
        let expected_balance: U256 = values.iter().sum();

        contract
            .sender(alice)
            ._mint_batch(
                alice,
                vec![token_id; 4],
                values.clone(),
                &vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should batch mint tokens");

        assert_eq!(
            expected_balance,
            contract.sender(alice).balance_of(alice, token_id)
        );

        let balances = contract
            .sender(alice)
            .balance_of_batch(vec![alice; 4], vec![token_id; 4])
            .motsu_expect("should return balances");

        assert_eq!(vec![expected_balance; 4], balances);
    }

    #[motsu::test]
    fn error_when_batch_mints_to_invalid_receiver(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);
        let values = random_values(1);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            ._mint_batch(
                invalid_receiver,
                token_ids,
                values,
                &vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not batch mint tokens for invalid receiver",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_batch_mints_not_equal_arrays(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(3);
        let values = random_values(4);

        let err = contract
            .sender(alice)
            ._mint_batch(alice, token_ids, values, &vec![0, 1, 2, 3].into())
            .motsu_expect_err(
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
    fn burns(contract: Contract<Erc1155>, alice: Address) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);

        let token_id = token_ids[0];
        let value = values[0];

        contract
            .sender(alice)
            ._burn(alice, token_id, value)
            .motsu_expect("should burn tokens");

        let balances = contract.sender(alice).balance_of(alice, token_id);

        assert_eq!(U256::ZERO, balances);
    }

    #[motsu::test]
    fn error_when_burns_from_invalid_sender(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);
        let invalid_sender = Address::ZERO;

        let err = contract
            .sender(alice)
            ._burn(invalid_sender, token_ids[0], values[0])
            .motsu_expect_err("should not burn token for invalid sender");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_burns_with_insufficient_balance(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);

        let err = contract
            .sender(alice)
            ._burn(alice, token_ids[0], values[0] + U256::ONE)
            .motsu_expect_err(
                "should not burn token when insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == alice && balance == values[0] && needed == values[0] + U256::ONE && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn burns_batch(contract: Contract<Erc1155>, alice: Address) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);

        contract
            .sender(alice)
            ._burn_batch(alice, token_ids.clone(), values.clone())
            .motsu_expect("should batch burn tokens");

        let balances = contract
            .sender(alice)
            .balance_of_batch(vec![alice; 4], token_ids.clone())
            .motsu_expect("should return balances");

        assert_eq!(vec![U256::ZERO; 4], balances);
    }

    #[motsu::test]
    fn burns_batch_same_token(contract: Contract<Erc1155>, alice: Address) {
        let token_id = U256::ONE;
        let value = uint!(80_U256);

        contract
            .sender(alice)
            ._mint(alice, token_id, value, &vec![0, 1, 2, 3].into())
            .motsu_expect("should mint token");

        contract
            .sender(alice)
            ._burn_batch(
                alice,
                vec![token_id; 4],
                vec![
                    uint!(20_U256),
                    uint!(10_U256),
                    uint!(30_U256),
                    uint!(20_U256),
                ],
            )
            .motsu_expect("should batch burn tokens");

        assert_eq!(
            U256::ZERO,
            contract.sender(alice).balance_of(alice, token_id)
        );
    }

    #[motsu::test]
    fn error_when_batch_burns_from_invalid_sender(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);
        let invalid_sender = Address::ZERO;

        let err = contract
            .sender(alice)
            ._burn_batch(invalid_sender, token_ids, values)
            .motsu_expect_err(
                "should not batch burn tokens for invalid sender",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_batch_burns_with_insufficient_balance(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);

        let err = contract
            .sender(alice)
            ._burn_batch(
                alice,
                token_ids.clone(),
                values.clone().into_iter().map(|x| x + U256::ONE).collect(),
            )
            .motsu_expect_err(
                "should not batch burn tokens when insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == alice && balance == values[0] && needed == values[0] + U256::ONE && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn error_when_batch_burns_not_equal_arrays(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 3);

        let err = contract
            .sender(alice)
            ._burn_batch(alice, token_ids, append(values, 4))
            .motsu_expect_err(
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
    fn safe_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 2);
        let amount_one = values[0] - U256::ONE;
        let amount_two = values[1] - U256::ONE;

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        contract
            .sender(alice)
            .safe_transfer_from(
                bob,
                dave,
                token_ids[0],
                amount_one,
                vec![].into(),
            )
            .motsu_expect("should transfer tokens from Alice to Bob");
        contract
            .sender(alice)
            .safe_transfer_from(
                bob,
                dave,
                token_ids[1],
                amount_two,
                vec![].into(),
            )
            .motsu_expect("should transfer tokens from Alice to Bob");

        let balance_id_one =
            contract.sender(alice).balance_of(dave, token_ids[0]);
        let balance_id_two =
            contract.sender(alice).balance_of(dave, token_ids[1]);

        assert_eq!(amount_one, balance_id_one);
        assert_eq!(amount_two, balance_id_two);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            .safe_transfer_from(
                alice,
                invalid_receiver,
                token_ids[0],
                values[0],
                vec![].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens to the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);
        let invalid_sender = Address::ZERO;

        contract
            .sender(invalid_sender)
            .set_approval_for_all(alice, true)
            .motsu_unwrap();

        let err = contract
            .sender(alice)
            .safe_transfer_from(
                invalid_sender,
                alice,
                token_ids[0],
                values[0],
                vec![].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens from the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);

        let err = contract
            .sender(bob)
            .safe_transfer_from(
                alice,
                bob,
                token_ids[0],
                values[0],
                vec![].into(),
            )
            .motsu_expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == bob && owner == alice
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);
        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        let err = contract
            .sender(alice)
            .safe_transfer_from(
                bob,
                dave,
                token_ids[0],
                values[0] + U256::ONE,
                vec![].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens with insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == bob && balance == values[0] && needed == values[0] + U256::ONE && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn safe_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        dave: Address,
        charlie: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(dave, 1);

        contract
            .sender(dave)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Dave's tokens to Alice");

        contract
            .sender(alice)
            .safe_transfer_from(
                dave,
                charlie,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should transfer tokens from Alice to Bob");

        let balance = contract.sender(alice).balance_of(charlie, token_ids[0]);

        assert_eq!(values[0], balance);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        dave: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(dave, 1);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            .do_safe_transfer_from(
                dave,
                invalid_receiver,
                token_ids,
                values,
                &vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens to the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);
        let invalid_sender = Address::ZERO;

        contract
            .sender(invalid_sender)
            .set_approval_for_all(alice, true)
            .motsu_unwrap();

        let err = contract
            .sender(alice)
            .safe_transfer_from(
                invalid_sender,
                alice,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens from the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);

        let err = contract
            .sender(bob)
            .safe_transfer_from(
                alice,
                bob,
                token_ids[0],
                values[0],
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == bob && owner == alice
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        let err = contract
            .sender(alice)
            .safe_transfer_from(
                bob,
                dave,
                token_ids[0],
                values[0] + U256::ONE,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens with insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == bob && balance == values[0] && needed == values[0] + U256::ONE && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn safe_batch_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(dave, 2);
        let amount_one = values[0] - U256::ONE;
        let amount_two = values[1] - U256::ONE;

        contract
            .sender(dave)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Dave's tokens to Alice");

        contract
            .sender(alice)
            .safe_batch_transfer_from(
                dave,
                bob,
                token_ids.clone(),
                vec![amount_one, amount_two],
                vec![].into(),
            )
            .motsu_expect("should transfer tokens from Alice to Bob");

        let balance_id_one =
            contract.sender(alice).balance_of(bob, token_ids[0]);
        let balance_id_two =
            contract.sender(alice).balance_of(bob, token_ids[1]);

        assert_eq!(amount_one, balance_id_one);
        assert_eq!(amount_two, balance_id_two);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_batch_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                alice,
                invalid_receiver,
                token_ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens to the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_batch_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);
        let invalid_sender = Address::ZERO;

        contract
            .sender(invalid_sender)
            .set_approval_for_all(alice, true)
            .motsu_unwrap();

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                invalid_sender,
                alice,
                token_ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens from the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_batch_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 2);

        let err = contract
            .sender(bob)
            .safe_batch_transfer_from(
                alice,
                bob,
                token_ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .motsu_expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == bob && owner == alice
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_batch_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(charlie, 2);

        contract
            .sender(charlie)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Charlie's tokens to Alice");

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                charlie,
                bob,
                token_ids.clone(),
                vec![values[0] + U256::ONE, values[1]],
                vec![].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens with insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == charlie && balance == values[0] && needed == values[0] + U256::ONE && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn error_when_not_equal_arrays_safe_batch_transfer_from(
        contract: Contract<Erc1155>,
        alice: Address,
        dave: Address,
        charlie: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);

        contract
            .sender(dave)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Dave's tokens to Alice");

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                dave,
                charlie,
                token_ids.clone(),
                append(values, 4),
                vec![].into(),
            )
            .motsu_expect_err(
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
    fn safe_batch_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(dave, 2);

        contract
            .sender(dave)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Dave's tokens to Alice");

        contract
            .sender(alice)
            .safe_batch_transfer_from(
                dave,
                bob,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should transfer tokens from Alice to Bob");

        let balance_id_one =
            contract.sender(alice).balance_of(bob, token_ids[0]);
        let balance_id_two =
            contract.sender(alice).balance_of(bob, token_ids[1]);

        assert_eq!(values[0], balance_id_one);
        assert_eq!(values[1], balance_id_two);
    }

    #[motsu::test]
    fn error_when_invalid_receiver_safe_batch_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                alice,
                invalid_receiver,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens to the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_safe_batch_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);
        let invalid_sender = Address::ZERO;

        contract
            .sender(invalid_sender)
            .set_approval_for_all(alice, true)
            .motsu_unwrap();

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                invalid_sender,
                alice,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens from the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_missing_approval_safe_batch_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 2);

        let err = contract
            .sender(bob)
            .safe_batch_transfer_from(
                alice,
                bob,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err("should not transfer tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                operator,
                owner
            }) if operator == bob && owner == alice
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_safe_batch_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
        charlie: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(charlie, 2);

        contract
            .sender(charlie)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Charlie's tokens to Alice");

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                charlie,
                bob,
                token_ids.clone(),
                vec![values[0] + U256::ONE, values[1]],
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer tokens with insufficient balance",
            );

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id
            }) if sender == charlie && balance == values[0] && needed == values[0] + U256::ONE && token_id == token_ids[0]
        ));
    }

    #[motsu::test]
    fn error_when_not_equal_arrays_safe_batch_transfer_from_with_data(
        contract: Contract<Erc1155>,
        alice: Address,
        dave: Address,
        charlie: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);

        contract
            .sender(dave)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Dave's tokens to Alice");

        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                dave,
                charlie,
                token_ids.clone(),
                append(values, 4),
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
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
        let actual = <Erc1155 as IErc1155>::interface_id();
        let expected: B32 = 0xd9b67a26_u32.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc1155>, alice: Address) {
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc1155 as IErc1155>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc1155 as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
}
