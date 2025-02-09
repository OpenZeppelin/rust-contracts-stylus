//! Implementation of the [`Erc721`] token standard.
use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, FixedBytes, U128, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    call::{self, Call, MethodError},
    evm, function_selector, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256},
};

use crate::utils::{
    introspection::erc165::{Erc165, IErc165},
    math::storage::{AddAssignUnchecked, SubAssignUnchecked},
};

pub mod extensions;
mod receiver;
pub use receiver::IERC721Receiver;

/// The expected value returned from [`IERC721Receiver::on_erc_721_received`].
pub const RECEIVER_FN_SELECTOR: [u8; 4] =
    function_selector!("onERC721Received", Address, Address, U256, Bytes,);

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the `token_id` token is transferred from `from` to `to`.
        ///
        /// * `from` - Address from which the token will be transferred.
        /// * `to` - Address where the token will be transferred to.
        /// * `token_id` - Token id as a number.
        #[allow(missing_docs)]
        event Transfer(
            address indexed from,
            address indexed to,
            uint256 indexed token_id
        );

        /// Emitted when `owner` enables `approved` to manage the `token_id` token.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `approved` - Address of the approver.
        /// * `token_id` - Token id as a number.
        #[allow(missing_docs)]
        event Approval(
            address indexed owner,
            address indexed approved,
            uint256 indexed token_id
        );

        /// Emitted when `owner` enables or disables (`approved`) `operator`
        /// to manage all of its assets.
        ///
        /// * `owner` - Address of the owner of the token.
        /// * `operator` - Address of an operator that
        ///   will manage operations on the token.
        /// * `approved` - Whether or not permission has been granted. If true,
        ///   this means `operator` will be allowed to manage `owner`'s assets.
        #[allow(missing_docs)]
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    }

    sol! {
        /// Indicates that an address can't be an owner.
        /// For example, `Address::ZERO` is a forbidden owner in [`Erc721`].
        /// Used in balance queries.
        ///
        /// * `owner` - The address deemed to be an invalid owner.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721InvalidOwner(address owner);

        /// Indicates a `token_id` whose `owner` is the zero address.
        ///
        /// * `token_id` - Token id as a number.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721NonexistentToken(uint256 token_id);

        /// Indicates an error related to the ownership over a particular token.
        /// Used in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        /// * `token_id` - Token id as a number.
        /// * `owner` - Address of the owner of the token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721IncorrectOwner(address sender, uint256 token_id, address owner);

        /// Indicates a failure with the token `sender`. Used in transfers.
        ///
        /// * `sender` - An address whose token is being transferred.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721InvalidSender(address sender);

        /// Indicates a failure with the token `receiver`. Used in transfers.
        ///
        /// * `receiver` - Address that receives the token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721InvalidReceiver(address receiver);

        /// Indicates a failure with the `operator`’s approval. Used in transfers.
        ///
        /// * `operator` - Address that may be allowed to operate on tokens
        ///   without being their owner.
        /// * `token_id` - Token id as a number.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721InsufficientApproval(address operator, uint256 token_id);

        /// Indicates a failure with the `approver` of a token to be approved.
        /// Used in approvals.
        ///
        /// * `approver` - Address initiating an approval operation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721InvalidApprover(address approver);

        /// Indicates a failure with the `operator` to be approved.
        /// Used in approvals.
        ///
        /// * `operator` - Address that may be allowed to operate on tokens
        ///   without being their owner.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721InvalidOperator(address operator);
    }
}

/// An [`Erc721`] error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates that an address can't be an owner.
    /// For example, `Address::ZERO` is a forbidden owner in [`Erc721`].
    /// Used in balance queries.
    InvalidOwner(ERC721InvalidOwner),
    /// Indicates a `token_id` whose `owner` is the zero address.
    NonexistentToken(ERC721NonexistentToken),
    /// Indicates an error related to the ownership over a particular token.
    /// Used in transfers.
    IncorrectOwner(ERC721IncorrectOwner),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(ERC721InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(ERC721InvalidReceiver),
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
    InsufficientApproval(ERC721InsufficientApproval),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(ERC721InvalidApprover),
    /// Indicates a failure with the `operator` to be approved. Used in
    /// approvals.
    InvalidOperator(ERC721InvalidOperator),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc721`] token.
#[storage]
pub struct Erc721 {
    /// Maps tokens to owners.
    #[allow(clippy::used_underscore_binding)]
    pub _owners: StorageMap<U256, StorageAddress>,
    /// Maps users to balances.
    #[allow(clippy::used_underscore_binding)]
    pub _balances: StorageMap<Address, StorageU256>,
    /// Maps tokens to approvals.
    #[allow(clippy::used_underscore_binding)]
    pub _token_approvals: StorageMap<U256, StorageAddress>,
    /// Maps owners to a mapping of operator approvals.
    #[allow(clippy::used_underscore_binding)]
    pub _operator_approvals:
        StorageMap<Address, StorageMap<Address, StorageBool>>,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc721 {}

/// Required interface of an [`Erc721`] compliant contract.
#[interface_id]
pub trait IErc721 {
    /// The error type associated to this ERC-721 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the number of tokens in `owner`'s account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidOwner`] - If owner address is `Address::ZERO`.
    fn balance_of(&self, owner: Address) -> Result<U256, Self::Error>;

    /// Returns the owner of the `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    fn owner_of(&self, token_id: U256) -> Result<Address, Self::Error>;

    /// Safely transfers `token_id` token from `from` to `to`, checking first
    /// that contract recipients are aware of the [`Erc721`] protocol to
    /// prevent tokens from being forever locked.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::IncorrectOwner`]  - If the previous owner is not `from`.
    /// * [`Error::InsufficientApproval`] - If the caller does not have the
    ///   right to approve.
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, `to` is `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error>;

    /// Safely transfers `token_id` token from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Erc721::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    ///  * [`Error::IncorrectOwner`] - If the previous owner is not `from`.
    ///  * [`Error::InsufficientApproval`] - If the caller does not have the
    ///    right to approve.
    ///  * [`Error::NonexistentToken`] - If the token does not exist.
    ///  * [`Error::InvalidReceiver`] - If
    ///    [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    ///    interface id or returned with error, or `to` is `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Self::Error>;

    /// Transfers `token_id` token from `from` to `to`.
    ///
    /// WARNING: Note that the caller is responsible to confirm that the
    /// recipient is capable of receiving [`Erc721`] or else they may be
    /// permanently lost. Usage of [`Self::safe_transfer_from`] prevents loss,
    /// though the caller must understand this adds an external call which
    /// potentially creates a reentrancy vulnerability, unless it is disabled.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    /// * [`Error::IncorrectOwner`] - If the previous owner is not `from`.
    /// * [`Error::InsufficientApproval`] - If the caller does not have the
    ///   right to approve.
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error>;

    /// Gives permission to `to` to transfer `token_id` token to another
    /// account. The approval is cleared when the token is transferred.
    ///
    /// Only a single account can be approved at a time,
    /// so approving the `Address::ZERO` clears previous approvals.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    /// * [`Error::InvalidApprover`] - If `auth` (param of [`Erc721::_approve`])
    ///   does not have a right to approve this token.
    ///
    /// # Events
    ///
    /// * [`Approval`].
    fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error>;

    /// Approve or remove `operator` as an operator for the caller.
    ///
    /// Operators can call [`Self::transfer_from`] or
    /// [`Self::safe_transfer_from`] for any token owned by the caller.
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

    /// Returns the account approved for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    fn get_approved(&self, token_id: U256) -> Result<Address, Self::Error>;

    /// Returns whether the `operator` is allowed to manage all the assets of
    /// `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `operator` - Account to be checked.
    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool;
}

#[public]
impl IErc721 for Erc721 {
    type Error = Error;

    fn balance_of(&self, owner: Address) -> Result<U256, Error> {
        if owner.is_zero() {
            return Err(ERC721InvalidOwner { owner: Address::ZERO }.into());
        }
        Ok(self._balances.get(owner))
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, Error> {
        self._require_owned(token_id)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        self.safe_transfer_from_with_data(from, to, token_id, vec![].into())
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.transfer_from(from, to, token_id)?;
        self._check_on_erc721_received(msg::sender(), from, to, token_id, &data)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        // Setting an "auth" argument enables the `_is_authorized` check which
        // verifies that the token exists (`from != 0`). Therefore, it is
        // not needed to verify that the return value is not 0 here.
        let previous_owner = self._update(to, token_id, msg::sender())?;
        if previous_owner != from {
            return Err(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            }
            .into());
        }
        Ok(())
    }

    fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self._approve(to, token_id, msg::sender(), true)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        self._set_approval_for_all(msg::sender(), operator, approved)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, Error> {
        self._require_owned(token_id)?;
        Ok(self._get_approved(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self._operator_approvals.get(owner).get(operator)
    }
}

impl IErc165 for Erc721 {
    fn supports_interface(interface_id: FixedBytes<4>) -> bool {
        <Self as IErc721>::INTERFACE_ID == u32::from_be_bytes(*interface_id)
            || Erc165::supports_interface(interface_id)
    }
}

impl Erc721 {
    /// Returns the owner of the `token_id`. Does NOT revert if the token
    /// doesn't exist.
    ///
    /// IMPORTANT: Any overrides to this function that add ownership of tokens
    /// not tracked by the core [`Erc721`] logic MUST be matched with the use
    /// of [`Self::_increase_balance`] to keep balances consistent with
    /// ownership. The invariant to preserve is that for any address `a` the
    /// value returned by [`Self::balance_of(a)`] must be equal to the number of
    /// tokens such that [`Self::_owner_of(token_id)`] is `a`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    #[must_use]
    pub fn _owner_of(&self, token_id: U256) -> Address {
        self._owners.get(token_id)
    }

    /// Returns the approved address for `token_id`.
    /// Returns 0 if `token_id` is not minted.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    #[must_use]
    pub fn _get_approved(&self, token_id: U256) -> Address {
        self._token_approvals.get(token_id)
    }

    /// Returns whether `spender` is allowed to manage `owner`'s tokens, or
    /// `token_id` in particular (ignoring whether it is owned by `owner`).
    ///
    /// WARNING: This function assumes that `owner` is the actual owner of
    /// `token_id` and does not verify this assumption.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `spender` - Account that will spend token.
    /// * `token_id` - Token id as a number.
    #[must_use]
    pub fn _is_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> bool {
        !spender.is_zero()
            && (owner == spender
                || self.is_approved_for_all(owner, spender)
                || self._get_approved(token_id) == spender)
    }

    /// Checks if `operator` can operate on `token_id`, assuming the provided
    /// `owner` is the actual owner. Reverts if:
    /// - `operator` does not have approval from `owner` for `token_id`.
    /// - `operator` does not have approval to manage all of `owner`'s assets.
    ///
    /// WARNING: This function assumes that `owner` is the actual owner of
    /// `token_id` and does not verify this assumption.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `operator` - Account that will spend token.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    /// * [`Error::InsufficientApproval`] - If `spender` does not have the right
    ///   to approve.
    pub fn _check_authorized(
        &self,
        owner: Address,
        operator: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if self._is_authorized(owner, operator, token_id) {
            return Ok(());
        }

        if owner.is_zero() {
            Err(ERC721NonexistentToken { token_id }.into())
        } else {
            Err(ERC721InsufficientApproval { operator, token_id }.into())
        }
    }

    /// Unsafe write access to the balances, used by extensions that "mint"
    /// tokens using an [`Self::owner_of`] override.
    ///
    /// NOTE: the value is limited to type(uint128).max. This protects against
    /// _balance overflow. It is unrealistic that a `U256` would ever
    /// overflow from increments when these increments are bounded to `u128`
    /// values.
    ///
    /// WARNING: Increasing an account's balance using this function tends to
    /// be paired with an override of the [`Self::_owner_of`] function to
    /// resolve the ownership of the corresponding tokens so that balances and
    /// ownership remain consistent with one another.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Account to increase balance.
    /// * `value` - The number of tokens to increase balance.
    pub fn _increase_balance(&mut self, account: Address, value: U128) {
        self._balances.setter(account).add_assign_unchecked(U256::from(value));
    }

    /// Transfers `token_id` from its current owner to `to`, or alternatively
    /// mints (or burns) if the current owner (or `to`) is the `Address::ZERO`.
    /// Returns the owner of the `token_id` before the update.
    ///
    /// The `auth` argument is optional. If the value passed is non-zero, then
    /// this function will check that `auth` is either the owner of the
    /// token, or approved to operate on the token (by the owner).
    ///
    /// NOTE: If overriding this function in a way that tracks balances, see
    /// also [`Self::_increase_balance`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `auth` - Account used for authorization of the update.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist and `auth`
    ///   is not `Address::ZERO`.
    /// * [`Error::InsufficientApproval`] - If `auth` is not `Address::ZERO` and
    ///   `auth` does not have a right to approve this token.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let from = self._owner_of(token_id);

        // Perform (optional) operator check.
        if !auth.is_zero() {
            self._check_authorized(from, auth, token_id)?;
        }

        // Execute the update.
        if !from.is_zero() {
            // Clear approval. No need to re-authorize or emit the `Approval`
            // event.
            self._approve(Address::ZERO, token_id, Address::ZERO, false)?;
            self._balances.setter(from).sub_assign_unchecked(uint!(1_U256));
        }

        if !to.is_zero() {
            self._balances.setter(to).add_assign_unchecked(uint!(1_U256));
        }

        self._owners.setter(token_id).set(to);
        evm::log(Transfer { from, to, token_id });
        Ok(from)
    }

    /// Mints `token_id` and transfers it to `to`.
    ///
    /// WARNING: Usage of this method is discouraged, use [`Self::_safe_mint`]
    /// whenever possible.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `token_id` already exists.
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        if to.is_zero() {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if !previous_owner.is_zero() {
            return Err(ERC721InvalidSender { sender: Address::ZERO }.into());
        }
        Ok(())
    }

    /// Mints `token_id`, transfers it to `to`,
    /// and checks for `to`'s acceptance.
    ///
    /// An additional `data` parameter is forwarded to
    /// [`IERC721Receiver::on_erc_721_received`] to contract recipients.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Erc721::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If `token_id` already exists.
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC721Receiver::on_erc_721_received`] hasn't returned its interface
    ///   id or returned with an error.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _safe_mint(
        &mut self,
        to: Address,
        token_id: U256,
        data: &Bytes,
    ) -> Result<(), Error> {
        self._mint(to, token_id)?;
        self._check_on_erc721_received(
            msg::sender(),
            Address::ZERO,
            to,
            token_id,
            data,
        )
    }

    /// Destroys `token_id`.
    ///
    /// The approval is cleared when the token is burned. This is an
    /// internal function that does not check if the sender is authorized
    /// to operate on the token.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _burn(&mut self, token_id: U256) -> Result<(), Error> {
        let previous_owner =
            self._update(Address::ZERO, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(ERC721NonexistentToken { token_id }.into());
        }
        Ok(())
    }

    /// Transfers `token_id` from `from` to `to`.
    ///
    /// As opposed to [`Self::transfer_from`], this imposes no restrictions on
    /// `msg::sender`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    /// * [`Error::NonexistentToken`] - If `token_id` does not exist.
    /// * [`Error::IncorrectOwner`] - If the previous owner is not `from`.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(ERC721NonexistentToken { token_id }.into());
        } else if previous_owner != from {
            return Err(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            }
            .into());
        }

        Ok(())
    }

    /// Safely transfers `token_id` token from `from` to `to`, checking that
    /// contract recipients are aware of the [`Erc721`] standard to prevent
    /// tokens from being forever locked.
    ///
    /// `data` is additional data, it has
    /// no specified format and it is sent in call to `to`. This internal
    /// function is like [`Self::safe_transfer_from`] in the sense that it
    /// invokes [`IERC721Receiver::on_erc_721_received`] on the receiver,
    /// and can be used to e.g. implement alternative mechanisms to perform
    /// token transfer, such as signature-based.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Erc721::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If `to` is `Address::ZERO`.
    /// * [`Error::NonexistentToken`] - If `token_id` does not exist.
    /// * [`Error::IncorrectOwner`] - If the previous owner is not `from`.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _safe_transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: &Bytes,
    ) -> Result<(), Error> {
        self._transfer(from, to, token_id)?;
        self._check_on_erc721_received(msg::sender(), from, to, token_id, data)
    }

    /// Approve `to` to operate on `token_id`.
    ///
    /// The `auth` argument is optional. If the value passed is non 0, then this
    /// function will check that `auth` is either the owner of the token, or
    /// approved to operate on all tokens held by this owner.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `auth` - Account used for authorization of the update.
    /// * `emit_event` - Emit an [`Approval`] event flag.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If the token does not exist.
    /// * [`Error::InvalidApprover`] - If `auth` does not have a right to
    ///   approve this token.
    ///
    /// # Events
    ///
    /// * [`Approval`].
    pub fn _approve(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
        emit_event: bool,
    ) -> Result<(), Error> {
        // Avoid reading the owner unless necessary.
        if emit_event || !auth.is_zero() {
            let owner = self._require_owned(token_id)?;

            // We do not use [`Self::_is_authorized`] because single-token
            // approvals should not be able to call `approve`.
            if !auth.is_zero()
                && owner != auth
                && !self.is_approved_for_all(owner, auth)
            {
                return Err(ERC721InvalidApprover { approver: auth }.into());
            }

            if emit_event {
                evm::log(Approval { owner, approved: to, token_id });
            }
        }

        self._token_approvals.setter(token_id).set(to);
        Ok(())
    }

    /// Approve `operator` to operate on all of `owner`'s tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account the token's owner.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `approved` - Whether permission will be granted. If true, this means.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidOperator`] - If `operator` is `Address::ZERO`.
    ///
    /// # Events
    ///
    /// * [`ApprovalForAll`].
    pub fn _set_approval_for_all(
        &mut self,
        owner: Address,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        if operator.is_zero() {
            return Err(ERC721InvalidOperator { operator }.into());
        }

        self._operator_approvals.setter(owner).setter(operator).set(approved);
        evm::log(ApprovalForAll { owner, operator, approved });
        Ok(())
    }

    /// Reverts if the `token_id` doesn't have a current owner (it hasn't been
    /// minted, or it has been burned). Returns the owner.
    ///
    /// Overrides to ownership logic should be done to
    /// [`Self::_owner_of`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`Error::NonexistentToken`] - If token does not exist.
    pub fn _require_owned(&self, token_id: U256) -> Result<Address, Error> {
        let owner = self._owner_of(token_id);
        if owner.is_zero() {
            return Err(ERC721NonexistentToken { token_id }.into());
        }
        Ok(owner)
    }

    /// Performs an acceptance check for the provided `operator` by calling
    /// [`IERC721Receiver::on_erc_721_received`] on the `to` address. The
    /// `operator` is generally the address that initiated the token transfer
    /// (i.e. `msg::sender()`).
    ///
    /// The acceptance call is not executed and treated as a no-op if the
    /// target address doesn't contain code (i.e. an EOA). Otherwise, the
    /// recipient must implement [`IERC721Receiver::on_erc_721_received`] and
    /// return the acceptance magic value to accept the transfer.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IERC721Receiver::on_erc_721_received`] hasn't returned its interface
    ///   id or returned with error.
    pub fn _check_on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        to: Address,
        token_id: U256,
        data: &Bytes,
    ) -> Result<(), Error> {
        if !to.has_code() {
            return Ok(());
        }

        let receiver = IERC721Receiver::new(to);
        let call = Call::new_in(self);
        let result = receiver.on_erc_721_received(
            call,
            operator,
            from,
            token_id,
            data.to_vec().into(),
        );

        let id = match result {
            Ok(id) => id,
            Err(e) => {
                if let call::Error::Revert(ref reason) = e {
                    if !reason.is_empty() {
                        // Non-IERC721Receiver implementer.
                        return Err(e.into());
                    }
                }

                return Err(ERC721InvalidReceiver { receiver: to }.into());
            }
        };

        // Token rejected.
        if id != RECEIVER_FN_SELECTOR {
            return Err(ERC721InvalidReceiver { receiver: to }.into());
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use super::{
        ERC721IncorrectOwner, ERC721InsufficientApproval,
        ERC721InvalidApprover, ERC721InvalidOperator, ERC721InvalidOwner,
        ERC721InvalidReceiver, ERC721InvalidSender, ERC721NonexistentToken,
        Erc721, Error, IErc721,
    };
    use crate::utils::introspection::erc165::IErc165;

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn error_when_checking_balance_of_invalid_owner(contract: Erc721) {
        let invalid_owner = Address::ZERO;
        let err = contract
            .balance_of(invalid_owner)
            .expect_err("should return `Error::InvalidOwner`");
        assert!(matches!(
            err,
            Error::InvalidOwner(ERC721InvalidOwner { owner: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn balance_of_zero_balance(contract: Erc721) {
        let owner = msg::sender();
        let balance =
            contract.balance_of(owner).expect("should return `U256::ZERO`");
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn error_when_checking_owner_of_nonexistent_token(contract: Erc721) {
        let err = contract
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn mints(contract: Erc721) {
        let alice = msg::sender();

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        contract._mint(alice, TOKEN_ID).expect("should mint a token for Alice");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(initial_balance + uint!(1_U256), balance);
    }

    #[motsu::test]
    fn error_when_minting_token_id_twice(contract: Erc721) {
        let alice = msg::sender();
        contract
            ._mint(alice, TOKEN_ID)
            .expect("should mint the token a first time");
        let err = contract
            ._mint(alice, TOKEN_ID)
            .expect_err("should not mint a token with `TOKEN_ID` twice");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn error_when_minting_token_invalid_receiver(contract: Erc721) {
        let invalid_receiver = Address::ZERO;

        let err = contract
            ._mint(invalid_receiver, TOKEN_ID)
            .expect_err("should not mint a token for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn safe_mints(contract: Erc721) {
        let alice = msg::sender();

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        contract
            ._safe_mint(alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect("should mint a token for Alice");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(initial_balance + uint!(1_U256), balance);
    }

    #[motsu::test]
    fn error_when_safe_mint_token_id_twice(contract: Erc721) {
        let alice = msg::sender();
        contract
            ._mint(alice, TOKEN_ID)
            .expect("should mint the token a first time");

        let err = contract
            ._safe_mint(alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect_err("should not mint a token with `TOKEN_ID` twice");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn error_when_safe_mint_invalid_receiver(contract: Erc721) {
        let invalid_receiver = Address::ZERO;

        let err = contract
            ._safe_mint(invalid_receiver, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect_err("should not mint a token for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn transfers_from(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");
        contract
            .transfer_from(alice, BOB, TOKEN_ID)
            .expect("should transfer a token from Alice to Bob");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn transfers_from_approved_token(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        contract._token_approvals.setter(TOKEN_ID).set(alice);
        contract
            .transfer_from(BOB, alice, TOKEN_ID)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_from_approved_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");

        // As we cannot change `msg::sender`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert!(approved_for_all);

        contract
            .transfer_from(BOB, alice, TOKEN_ID)
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_to_invalid_receiver(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            .transfer_from(alice, invalid_receiver, TOKEN_ID)
            .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_from_incorrect_owner(
        contract: Erc721,
    ) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            .transfer_from(DAVE, BOB, TOKEN_ID)
            .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_with_insufficient_approval(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        let err = contract
            .transfer_from(BOB, alice, TOKEN_ID)
            .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                    operator,
                    token_id: t_id,
            }) if operator == alice && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_nonexistent_token(contract: Erc721) {
        let alice = msg::sender();
        let err = contract
            .transfer_from(alice, BOB, TOKEN_ID)
            .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                    token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn safe_transfers_from(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        contract
            .safe_transfer_from(alice, BOB, TOKEN_ID)
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");

        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_token(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        contract._token_approvals.setter(TOKEN_ID).set(alice);
        contract
            .safe_transfer_from(BOB, alice, TOKEN_ID)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert!(approved_for_all);

        contract
            .safe_transfer_from(BOB, alice, TOKEN_ID)
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_to_invalid_receiver(contract: Erc721) {
        let alice = msg::sender();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            .safe_transfer_from(alice, invalid_receiver, TOKEN_ID)
            .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_from_incorrect_owner(
        contract: Erc721,
    ) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            .safe_transfer_from(DAVE, BOB, TOKEN_ID)
            .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                owner,
                sender,
                token_id: t_id
            }) if sender == DAVE && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_with_insufficient_approval(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        let err = contract
            .safe_transfer_from(BOB, alice, TOKEN_ID)
            .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id
            }) if operator == alice && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_nonexistent_token(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        let err = contract
            .safe_transfer_from(alice, BOB, TOKEN_ID)
            .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn safe_transfers_from_with_data(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        contract
            .safe_transfer_from_with_data(
                alice,
                BOB,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");

        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn safe_transfers_from_with_data_approved_token(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        contract._token_approvals.setter(TOKEN_ID).set(alice);
        contract
            .safe_transfer_from_with_data(
                BOB,
                alice,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_from_with_data_approved_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert!(approved_for_all);

        contract
            .safe_transfer_from_with_data(
                BOB,
                alice,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_to_invalid_receiver(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            .safe_transfer_from_with_data(
                alice,
                invalid_receiver,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_from_incorrect_owner(
        contract: Erc721,
    ) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            .safe_transfer_from_with_data(
                DAVE,
                BOB,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == TOKEN_ID && owner == alice

        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .expect("should return the owner of the token");
        //
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_with_insufficient_approval(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        let err = contract
            .safe_transfer_from_with_data(
                BOB,
                alice,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id,
            }) if operator == alice && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_nonexistent_token(
        contract: Erc721,
    ) {
        let alice = msg::sender();
        let err = contract
            .safe_transfer_from_with_data(
                alice,
                BOB,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn approves(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .approve(BOB, TOKEN_ID)
            .expect("should approve Bob for operations on token");
        assert_eq!(contract._token_approvals.get(TOKEN_ID), BOB);
    }

    #[motsu::test]
    fn error_when_approve_for_nonexistent_token(contract: Erc721) {
        let err = contract
            .approve(BOB, TOKEN_ID)
            .expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_by_invalid_approver(contract: Erc721) {
        contract._mint(BOB, TOKEN_ID).expect("should mint a token");

        let err = contract
            .approve(DAVE, TOKEN_ID)
            .expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::InvalidApprover(ERC721InvalidApprover {
                approver
            }) if approver == msg::sender()
        ));
    }

    #[motsu::test]
    fn approval_for_all(contract: Erc721) {
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
    fn error_when_approval_for_all_for_invalid_operator(contract: Erc721) {
        let invalid_operator = Address::ZERO;

        let err = contract
            .set_approval_for_all(invalid_operator, true)
            .expect_err("should not approve for all for invalid operator");

        assert!(matches!(
            err,
            Error::InvalidOperator(ERC721InvalidOperator {
                operator
            }) if operator == invalid_operator
        ));
    }

    #[motsu::test]
    fn error_when_get_approved_of_nonexistent_token(contract: Erc721) {
        let err = contract
            .get_approved(TOKEN_ID)
            .expect_err("should not return approved for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn owner_of_works(contract: Erc721) {
        contract._mint(BOB, TOKEN_ID).expect("should mint a token");

        let owner = contract._owner_of(TOKEN_ID);
        assert_eq!(BOB, owner);
    }

    #[motsu::test]
    fn owner_of_nonexistent_token(contract: Erc721) {
        let owner = contract._owner_of(TOKEN_ID);
        assert_eq!(Address::ZERO, owner);
    }

    #[motsu::test]
    fn get_approved_nonexistent_token(contract: Erc721) {
        let approved = contract._get_approved(TOKEN_ID);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn get_approved_token_without_approval(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        let approved = contract._get_approved(TOKEN_ID);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn get_approved_token_with_approval(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .approve(BOB, TOKEN_ID)
            .expect("should approve Bob for operations on token");

        let approved = contract._get_approved(TOKEN_ID);
        assert_eq!(BOB, approved);
    }

    #[motsu::test]
    fn get_approved_token_with_approval_for_all(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");

        let approved = contract._get_approved(TOKEN_ID);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn is_authorized_nonexistent_token(contract: Erc721) {
        let alice = msg::sender();
        let authorized = contract._is_authorized(alice, BOB, TOKEN_ID);
        assert!(!authorized);
    }

    #[motsu::test]
    fn is_authorized_token_owner(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");

        let authorized = contract._is_authorized(alice, alice, TOKEN_ID);
        assert!(authorized);
    }

    #[motsu::test]
    fn is_authorized_without_approval(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");

        let authorized = contract._is_authorized(alice, BOB, TOKEN_ID);
        assert!(!authorized);
    }

    #[motsu::test]
    fn is_authorized_with_approval(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .approve(BOB, TOKEN_ID)
            .expect("should approve Bob for operations on token");

        let authorized = contract._is_authorized(alice, BOB, TOKEN_ID);
        assert!(authorized);
    }

    #[motsu::test]
    fn is_authorized_with_approval_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");

        let authorized = contract._is_authorized(alice, BOB, TOKEN_ID);
        assert!(authorized);
    }

    #[motsu::test]
    fn check_authorized_nonexistent_token(contract: Erc721) {
        let alice = msg::sender();
        let err = contract
            ._check_authorized(Address::ZERO, alice, TOKEN_ID)
            .expect_err("should not pass for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn check_authorized_token_owner(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");

        let result = contract._check_authorized(alice, alice, TOKEN_ID);

        assert!(result.is_ok());
    }

    #[motsu::test]
    fn check_authorized_without_approval(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");

        let err = contract
            ._check_authorized(alice, BOB, TOKEN_ID)
            .expect_err("should not pass without approval");

        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id
            }) if operator == BOB && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn check_authorized_with_approval(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .approve(BOB, TOKEN_ID)
            .expect("should approve Bob for operations on token");

        let result = contract._check_authorized(alice, BOB, TOKEN_ID);
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn check_authorized_with_approval_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");

        let result = contract._check_authorized(alice, BOB, TOKEN_ID);
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn burns(contract: Erc721) {
        let alice = msg::sender();
        let one = uint!(1_U256);

        contract._mint(alice, TOKEN_ID).expect("should mint a token for Alice");

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let result = contract._burn(TOKEN_ID);
        let balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let err = contract
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
                err,
                Error::NonexistentToken (ERC721NonexistentToken{
                    token_id: t_id
                }) if t_id == TOKEN_ID
        ));

        assert!(result.is_ok());

        assert_eq!(initial_balance - one, balance);
    }

    #[motsu::test]
    fn error_when_get_approved_of_previous_approval_burned(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token for Alice");
        contract
            .approve(BOB, TOKEN_ID)
            .expect("should approve a token for Bob");

        contract._burn(TOKEN_ID).expect("should burn previously minted token");

        let err = contract
            .get_approved(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_burn_nonexistent_token(contract: Erc721) {
        let err = contract
            ._burn(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn transfers(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");
        contract
            ._transfer(alice, BOB, TOKEN_ID)
            .expect("should transfer a token from Alice to Bob");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn transfers_approved_token(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        contract._token_approvals.setter(TOKEN_ID).set(alice);
        contract
            ._transfer(BOB, alice, TOKEN_ID)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_approved_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");

        // As we cannot change `msg::sender`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert!(approved_for_all);

        contract
            ._transfer(BOB, alice, TOKEN_ID)
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_to_invalid_receiver(contract: Erc721) {
        let alice = msg::sender();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            ._transfer(alice, invalid_receiver, TOKEN_ID)
            .expect_err("should not transfer to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_from_incorrect_owner(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            ._transfer(DAVE, BOB, TOKEN_ID)
            .expect_err("should not transfer from incorrect owner");

        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_nonexistent_token(contract: Erc721) {
        let alice = msg::sender();
        let err = contract
            ._transfer(alice, BOB, TOKEN_ID)
            .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn safe_transfers_internal(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        contract
            ._safe_transfer(alice, BOB, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");

        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn safe_transfers_internal_approved_token(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");
        contract._token_approvals.setter(TOKEN_ID).set(alice);
        contract
            ._safe_transfer(BOB, alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_internal_approved_for_all(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint token to Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert!(approved_for_all);

        contract
            ._safe_transfer(BOB, alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_to_invalid_receiver(contract: Erc721) {
        let alice = msg::sender();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            ._safe_transfer(
                alice,
                invalid_receiver,
                TOKEN_ID,
                &vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(TOKEN_ID)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_from_incorrect_owner(
        contract: Erc721,
    ) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token to Alice");

        let err = contract
            ._safe_transfer(DAVE, BOB, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_nonexistent_token(contract: Erc721) {
        let alice = msg::sender();
        let err = contract
            ._safe_transfer(alice, BOB, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .expect_err("should not transfer a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn approves_internal(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(alice, TOKEN_ID).expect("should mint a token");
        contract
            ._approve(BOB, TOKEN_ID, alice, false)
            .expect("should approve Bob for operations on token");
        assert_eq!(contract._token_approvals.get(TOKEN_ID), BOB);
    }

    #[motsu::test]
    fn error_when_approve_internal_for_nonexistent_token(contract: Erc721) {
        let err = contract
            ._approve(BOB, TOKEN_ID, msg::sender(), false)
            .expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_internal_by_invalid_approver(contract: Erc721) {
        let alice = msg::sender();
        contract._mint(BOB, TOKEN_ID).expect("should mint a token");

        let err = contract
            ._approve(DAVE, TOKEN_ID, alice, false)
            .expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::InvalidApprover(ERC721InvalidApprover {
                approver
            }) if approver == alice
        ));
    }

    #[motsu::test]
    fn approval_for_all_internal(contract: Erc721) {
        let alice = msg::sender();
        contract._operator_approvals.setter(alice).setter(BOB).set(false);

        contract
            ._set_approval_for_all(alice, BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");
        assert!(contract.is_approved_for_all(alice, BOB));

        contract._set_approval_for_all(alice, BOB, false).expect(
            "should disapprove Bob for operations on all Alice's tokens",
        );
        assert!(!contract.is_approved_for_all(alice, BOB));
    }

    #[motsu::test]
    fn error_when_approval_for_all_internal_for_invalid_operator(
        contract: Erc721,
    ) {
        let invalid_operator = Address::ZERO;

        let err = contract
            ._set_approval_for_all(msg::sender(), invalid_operator, true)
            .expect_err("should not approve for all for invalid operator");

        assert!(matches!(
            err,
            Error::InvalidOperator(ERC721InvalidOperator {
                operator
            }) if operator == invalid_operator
        ));
    }

    #[motsu::test]
    fn require_owned_works(contract: Erc721) {
        contract._mint(BOB, TOKEN_ID).expect("should mint a token");

        let owner = contract
            ._require_owned(TOKEN_ID)
            .expect("should return the owner of the token");

        assert_eq!(BOB, owner);
    }

    #[motsu::test]
    fn error_when_require_owned_for_nonexistent_token(contract: Erc721) {
        let err = contract
            ._require_owned(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721 as IErc721>::INTERFACE_ID;
        let expected = 0x80ac58cd;
        assert_eq!(actual, expected);

        let actual = <Erc721 as IErc165>::INTERFACE_ID;
        let expected = 0x01ffc9a7;
        assert_eq!(actual, expected);
    }
}
