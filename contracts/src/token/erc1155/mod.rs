//! Implementation of the [`Erc1155`] token standard.
use alloc::vec::Vec;

use alloy_primitives::{fixed_bytes, Address, FixedBytes, Uint, U256};
use stylus_sdk::{
    abi::Bytes,
    alloy_sol_types::sol,
    call::{self, Call, MethodError},
    evm, msg,
    prelude::*,
};

use crate::utils::math::storage::{AddAssignUnchecked, SubAssignUnchecked};

pub mod extensions;

sol! {
    /// Emitted when `value` amount of tokens of type `token_id` are transferred from `from` to `to` by `operator`.
    #[allow(missing_docs)]
    event TransferSingle(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256 token_id,
        uint256 value
    );

    /// Equivalent to multiple {TransferSingle} events, where `operator`.
    /// `from` and `to` are the same for all transfers.
    #[allow(missing_docs)]
    event TransferBatch(
        address indexed operator,
        address indexed from,
        address indexed to,
        uint256[] token_ids,
        uint256[] values
    );

    /// Emitted when `account` grants or revokes permission to `operator` to transfer their tokens, according to
    /// `approved`.
    #[allow(missing_docs)]
    event ApprovalForAll(address indexed account, address indexed operator, bool approved);

    /// Emitted when the URI for token type `token_id` changes to `value`, if it is a non-programmatic URI.
    ///
    /// If an {URI} event was emitted for `token_id`, the [standard]
    /// (https://eips.ethereum.org/EIPS/eip-1155#metadata-extensions[guarantees]) that `value` will equal the value
    /// returned by [`IERC1155MetadataURI-uri`].
    #[allow(missing_docs)]
    event URI(string value, uint256 indexed token_id);
}

sol! {
    /// Indicates an error related to the current `balance` of a `sender`. Used
    /// in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    /// * `balance` - Current balance for the interacting account.
    /// * `needed` - Minimum amount required to perform a transfer.
    /// * `token_id` - Identifier number of a token.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC1155InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 token_id);

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

    /// Indicates an array length mismatch between token_ids and values in a safeBatchTransferFrom operation.
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
    /// Indicates an error related to the current `balance` of `sender`. Used
    /// in transfers.
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
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(ERC1155InvalidApprover),
    /// Indicates a failure with the `operator` to be approved. Used in
    /// approvals.
    InvalidOperator(ERC1155InvalidOperator),
    /// Indicates an array length mismatch between token_ids and values in a
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
        /// * `token_id` - The ID of the token being transferred
        /// * `value` - The amount of tokens being transferred
        /// * `data` - Additional data with no specified format
        /// Return `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))` if transfer is allowed
        #[allow(missing_docs)]
        function onERC1155Received(
            address operator,
            address from,
            uint256 token_id,
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
        /// * `token_ids` - An array containing ids of each token being transferred (order and length must match values array)
        /// * `values` - An array containing amounts of each token being transferred (order and length must match ids array)
        /// * `data` - Additional data with no specified format
        /// * Return `bytes4(keccak256("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"))` if transfer is allowed
        #[allow(missing_docs)]
        function onERC1155BatchReceived(
            address operator,
            address from,
            uint256[] calldata token_ids,
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
    fn balance_of(
        &self,
        account: Address,
        token_id: U256,
    ) -> Result<U256, Self::Error>;

    /// xref:ROOT:erc1155.adoc#batch-operations[Batched] version of {balanceOf}.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `accounts` - All account of the tokens' owner.
    /// * `token_ids` - All token identifiers.
    ///
    /// Requirements:
    ///
    /// * - `accounts` and `token_ids` must have the same length.
    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        token_ids: Vec<U256>,
    ) -> Result<Vec<U256>, Self::Error>;

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
    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool;

    /// Transfers a `value` amount of tokens of type `token_id` from `from` to
    /// `to`.
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
    /// * - `from` must have a balance of tokens of type `token_id` of at least
    ///   `value` amount.
    /// * - If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_received`] and return the
    /// acceptance magic value.
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
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
    /// * - `token_ids` and `values` must have the same length.
    /// * - If `to` refers to a smart contract, it must implement
    ///   [`IERC1155Receiver::on_erc_1155_batch_received`] and return the
    /// acceptance magic value.
    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error>;
}

#[external]
impl IErc1155 for Erc1155 {
    type Error = Error;

    fn balance_of(
        &self,
        account: Address,
        token_id: U256,
    ) -> Result<U256, Self::Error> {
        Ok(self._balances.get(token_id).get(account))
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        token_ids: Vec<U256>,
    ) -> Result<Vec<U256>, Self::Error> {
        if accounts.len() != token_ids.len() {
            return Err(Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length: Uint::<256, 4>::from(token_ids.len()),
                values_length: Uint::<256, 4>::from(accounts.len()),
            }));
        }

        let mut balances = Vec::with_capacity(accounts.len());
        for (i, &account) in accounts.iter().enumerate() {
            let token_id = token_ids[i];
            let balance = self._balances.get(token_id).get(account);
            balances.push(balance);
        }
        Ok(balances)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool {
        self._operator_approvals.get(account).get(operator)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Erc1155 {
    /// Transfers a `value` amount of tokens of type `token_ids` from `from` to
    /// `to`. Will mint (or burn) if `from` (or `to`) is the zero address.
    ///
    /// Requirements:
    ///
    /// * - If `to` refers to a smart contract, it must implement either
    ///   [`IERC1155Receiver::on_erc_1155_received`]
    /// * or [`IERC1155Receiver::on_erc_1155_received`] and return the
    ///   acceptance magic value.
    /// * - `token_ids` and `values` must have the same length.
    ///
    /// /// # Errors
    ///
    /// If length of `token_ids` is not equal to length of `values`, then the
    /// error [`Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`Error::InsufficientBalance`] is returned.
    ///
    /// NOTE: The ERC-1155 acceptance check is not performed in this function.
    /// See [`Self::_updateWithAcceptanceCheck`] instead.
    ///
    /// Event
    ///
    /// Emits a [`TransferSingle`] event if the arrays contain one element, and
    /// [`TransferBatch`] otherwise.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Error> {
        if token_ids.len() != values.len() {
            return Err(Error::InvalidArrayLength(ERC1155InvalidArrayLength {
                ids_length: Uint::<256, 4>::from(token_ids.len()),
                values_length: Uint::<256, 4>::from(values.len()),
            }));
        }

        let operator = msg::sender();
        for (i, &token_id) in token_ids.iter().enumerate() {
            let value = values[i];
            let from_balance = self._balances.get(token_id).get(from);
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

            if to != Address::ZERO {
                self._balances
                    .setter(token_id)
                    .setter(to)
                    .add_assign_unchecked(value);
            }
        }

        if token_ids.len() == 1 {
            evm::log(TransferSingle {
                operator,
                from,
                to,
                token_id: token_ids[0],
                value: values[0],
            });
        } else {
            evm::log(TransferBatch { operator, from, to, token_ids, values });
        }

        Ok(())
    }

    /// Version of [`Self::_update`] that performs the token acceptance check by
    /// calling [`IERC1155Receiver::on_erc_1155_received`] or
    /// [`IERC1155Receiver::on_erc_1155_received`] on the receiver address if it
    /// contains code (eg. is a smart contract at the moment of execution).
    ///
    /// IMPORTANT: Overriding this function is discouraged because it poses a
    /// reentrancy risk from the receiver. So any update to the contract
    /// state after this function would break the check-effect-interaction
    /// pattern. Consider overriding [`Self::_update`] instead.
    ///
    /// # Arguments
    ///
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    fn _update_with_acceptance_check(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        self._update(from, to, token_ids.clone(), values.clone())?;
        if to != Address::ZERO {
            let operator = msg::sender();
            if token_ids.len() == 1 {
                let token_id = token_ids[0];
                let value = values[0];
                self._check_on_erc1155_received(
                    operator, from, to, token_id, value, data,
                )?;
            } else {
                self._check_on_erc1155_batch_received(
                    operator, from, to, token_ids, values, data,
                )?;
            }
        }
        Ok(())
    }

    /// Performs an acceptance check for the provided `operator` by
    /// calling [`IERC1155Receiver::on_erc_1155_received`] on the `to` address.
    /// The `operator` is generally the address that initiated the token
    /// transfer (i.e. `msg.sender`).
    ///
    /// The acceptance call is not executed and treated as a no-op if the
    /// target address is doesn't contain code (i.e. an EOA). Otherwise,
    /// the recipient must implement [`IERC1155Receiver::on_erc_1155_received`]
    /// and return the acceptance magic value to accept the transfer.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `value` - Amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    fn _check_on_erc1155_received(
        &mut self,
        operator: Address,
        from: Address,
        to: Address,
        token_id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        const RECEIVER_FN_SELECTOR: FixedBytes<4> = fixed_bytes!("f23a6e61");

        if !to.has_code() {
            return Ok(());
        }

        let receiver = IERC1155Receiver::new(to);
        let call = Call::new_in(self);
        let data = data.to_vec();
        let result = receiver
            .on_erc_1155_received(call, operator, from, token_id, value, data);

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
        if id != RECEIVER_FN_SELECTOR {
            return Err(ERC1155InvalidReceiver { receiver: to }.into());
        }

        Ok(())
    }

    /// Performs a batch acceptance check for the provided `operator` by
    /// calling [`IERC1155Receiver::on_erc_1155_received`] on the `to` address.
    /// The `operator` is generally the address that initiated the token
    /// transfer (i.e. `msg.sender`).
    ///
    /// The acceptance call is not executed and treated as a no-op if the
    /// target address is doesn't contain code (i.e. an EOA). Otherwise,
    /// the recipient must implement [`IERC1155Receiver::on_erc_1155_received`]
    /// and return the acceptance magic value to accept the transfer.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    fn _check_on_erc1155_batch_received(
        &mut self,
        operator: Address,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), Error> {
        const RECEIVER_FN_SELECTOR: FixedBytes<4> = fixed_bytes!("bc197c81");

        if !to.has_code() {
            return Ok(());
        }

        let receiver = IERC1155Receiver::new(to);
        let call = Call::new_in(self);
        let data = data.to_vec();
        let result = receiver.on_erc_1155_batch_received(
            call, operator, from, token_ids, values, data,
        );

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
        if id != RECEIVER_FN_SELECTOR {
            return Err(ERC1155InvalidReceiver { receiver: to }.into());
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use alloy_sol_types::token;
    use stylus_sdk::{contract, msg};

    use super::{ERC1155InvalidArrayLength, Erc1155, Error, IErc1155};

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");
    const CHARLIE: Address =
        address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u32>())).collect()
    }

    #[motsu::test]
    fn test_balance_of_zero_balance(contract: Erc1155) {
        let owner = msg::sender();
        let token_id = random_token_ids(1)[0];
        let balance = contract
            .balance_of(owner, token_id)
            .expect("should return `U256::ZERO`");
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn test_error_array_length_mismatch(contract: Erc1155) {
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
                ids_length,
                values_length: accounts_length,
            })
        ));
    }

    #[motsu::test]
    fn test_balance_of_batch_zero_balance(contract: Erc1155) {
        let token_ids = random_token_ids(4);
        let accounts = vec![ALICE, BOB, DAVE, CHARLIE];
        let balances = contract
            .balance_of_batch(accounts, token_ids)
            .expect("should return a vector of `U256::ZERO`");

        for balance in balances {
            assert_eq!(U256::ZERO, balance);
        }
    }
}
