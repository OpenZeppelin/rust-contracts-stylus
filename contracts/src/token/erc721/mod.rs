//! Implementation of the [`Erc721`] token standard.
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use alloy_primitives::{aliases::B32, Address, U128, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    call::{self, Call, MethodError},
    evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256},
};

use crate::utils::{
    introspection::erc165::IErc165,
    math::storage::{AddAssignUnchecked, SubAssignUnchecked},
};

pub mod abi;
pub mod extensions;
pub mod receiver;
pub mod utils;

pub use abi::Erc721ReceiverInterface;
pub use receiver::{IErc721Receiver, RECEIVER_FN_SELECTOR};
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
        #[derive(Debug)]
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
        #[derive(Debug)]
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
        #[derive(Debug)]
        #[allow(missing_docs)]
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    }

    sol! {
        /// Indicates that an address can't be an owner.
        /// For example, [`Address::ZERO`] is a forbidden owner in [`Erc721`].
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

        /// Indicates a failure with the receiver reverting with a reason.
        ///
        /// * `reason` - Revert reason.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error InvalidReceiverWithReason(string reason);
    }
}

/// An [`Erc721`] error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates that an address can't be an owner.
    /// For example, [`Address::ZERO`] is a forbidden owner in [`Erc721`].
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
    InvalidReceiverWithReason(InvalidReceiverWithReason),
    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    InsufficientApproval(ERC721InsufficientApproval),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(ERC721InvalidApprover),
    /// Indicates a failure with the `operator` to be approved. Used in
    /// approvals.
    InvalidOperator(ERC721InvalidOperator),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc721`] token.
#[storage]
pub struct Erc721 {
    /// Maps tokens to owners.
    pub(crate) owners: StorageMap<U256, StorageAddress>,
    /// Maps users to balances.
    pub(crate) balances: StorageMap<Address, StorageU256>,
    /// Maps tokens to approvals.
    pub(crate) token_approvals: StorageMap<U256, StorageAddress>,
    /// Maps owners to a mapping of operator approvals.
    pub(crate) operator_approvals:
        StorageMap<Address, StorageMap<Address, StorageBool>>,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc721 {}

/// Required interface of an [`Erc721`] compliant contract.
#[interface_id]
pub trait IErc721: IErc165 {
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
    /// * [`Error::InvalidOwner`] - If owner address is [`Address::ZERO`].
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
    ///   [`IErc721Receiver::on_erc721_received`] hasn't returned its
    /// interface id or returned with error, `to` is [`Address::ZERO`].
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
    ///    [`IErc721Receiver::on_erc721_received`] hasn't returned its interface
    ///    id or returned with error, or `to` is [`Address::ZERO`].
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
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
    /// so approving the [`Address::ZERO`] clears previous approvals.
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
#[implements(IErc721<Error = Error>, IErc165)]
impl Erc721 {}

#[public]
impl IErc721 for Erc721 {
    type Error = Error;

    fn balance_of(&self, owner: Address) -> Result<U256, Self::Error> {
        if owner.is_zero() {
            return Err(ERC721InvalidOwner { owner: Address::ZERO }.into());
        }
        Ok(self.balances.get(owner))
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, Self::Error> {
        self._require_owned(token_id)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error> {
        self.safe_transfer_from_with_data(from, to, token_id, vec![].into())
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.transfer_from(from, to, token_id)?;
        self._check_on_erc721_received(msg::sender(), from, to, token_id, &data)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error> {
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

    fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error> {
        self._approve(to, token_id, msg::sender(), true)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error> {
        self._set_approval_for_all(msg::sender(), operator, approved)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, Self::Error> {
        self._require_owned(token_id)?;
        Ok(self._get_approved(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.operator_approvals.get(owner).get(operator)
    }
}

#[public]
impl IErc165 for Erc721 {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc721>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
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
    /// value returned by [`Self::balance_of(a)`][Self::balance_of] must be
    /// equal to the number of tokens such that
    /// [`Self::_owner_of(token_id)`][Self::_owner_of] is `a`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    #[must_use]
    pub fn _owner_of(&self, token_id: U256) -> Address {
        self.owners.get(token_id)
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
        self.token_approvals.get(token_id)
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
    /// NOTE: the value is limited to [`U128::MAX`]. This protects against
    /// balance overflow. It is unrealistic that a [`U256`] would ever overflow
    /// from increments when these increments are bounded to [`U128`] values.
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
        self.balances.setter(account).add_assign_unchecked(U256::from(value));
    }

    /// Transfers `token_id` from its current owner to `to`, or alternatively
    /// mints (or burns) if the current owner (or `to`) is the
    /// [`Address::ZERO`]. Returns the owner of the `token_id` before the
    /// update.
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
    ///   is not [`Address::ZERO`].
    /// * [`Error::InsufficientApproval`] - If `auth` is not [`Address::ZERO`]
    ///   and `auth` does not have a right to approve this token.
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
            self.balances.setter(from).sub_assign_unchecked(U256::ONE);
        }

        if !to.is_zero() {
            self.balances.setter(to).add_assign_unchecked(U256::ONE);
        }

        self.owners.setter(token_id).set(to);
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
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
    /// [`IErc721Receiver::on_erc721_received`] to contract recipients.
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If
    ///   [`IErc721Receiver::on_erc721_received`] hasn't returned its interface
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
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
    /// invokes [`IErc721Receiver::on_erc721_received`] on the receiver,
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
    /// * [`Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
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

        self.token_approvals.setter(token_id).set(to);
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
    /// * [`Error::InvalidOperator`] - If `operator` is [`Address::ZERO`].
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

        self.operator_approvals.setter(owner).setter(operator).set(approved);
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
    /// [`IErc721Receiver::on_erc721_received`] on the `to` address. The
    /// `operator` is generally the address that initiated the token transfer
    /// (i.e. `msg::sender()`).
    ///
    /// The acceptance call is not executed and treated as a no-op if the
    /// target address doesn't contain code (i.e. an EOA). Otherwise, the
    /// recipient must implement [`IErc721Receiver::on_erc721_received`] and
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
    ///   [`IErc721Receiver::on_erc721_received`] hasn't returned its interface
    ///   id or returned an error.
    /// * [`Error::InvalidReceiverWithReason`] - If
    ///   [`IErc721Receiver::on_erc721_received`] reverted with revert data.
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

        let receiver = Erc721ReceiverInterface::new(to);
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
                        return Err(Error::InvalidReceiverWithReason(
                            InvalidReceiverWithReason {
                                reason: String::from_utf8_lossy(reason)
                                    .to_string(),
                            },
                        ));
                    }
                }

                // Non [`IErc721Receiver`] implementer.
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{aliases::B32, fixed_bytes, uint, Address, U256};
    use motsu::prelude::*;
    use stylus_sdk::{abi::Bytes, prelude::*};

    use super::{
        ERC721IncorrectOwner, ERC721InsufficientApproval,
        ERC721InvalidApprover, ERC721InvalidOperator, ERC721InvalidOwner,
        ERC721InvalidReceiver, ERC721InvalidSender, ERC721NonexistentToken,
        Erc721, Error, IErc721,
    };
    use crate::{
        token::erc721::receiver::tests::{
            BadSelectorReceiver721, EmptyReasonReceiver721,
            RevertingReceiver721,
        },
        utils::introspection::erc165::IErc165,
    };

    const TOKEN_ID: U256 = U256::ONE;

    #[motsu::test]
    fn error_when_checking_balance_of_invalid_owner(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_owner = Address::ZERO;
        let err = contract
            .sender(alice)
            .balance_of(invalid_owner)
            .motsu_expect_err("should return `Error::InvalidOwner`");
        assert!(matches!(
            err,
            Error::InvalidOwner(ERC721InvalidOwner { owner: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn balance_of_zero_balance(contract: Contract<Erc721>, owner: Address) {
        let balance = contract
            .sender(owner)
            .balance_of(owner)
            .motsu_expect("should return `U256::ZERO`");
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn error_when_checking_owner_of_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn mints(contract: Contract<Erc721>, alice: Address) {
        let initial_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token for Alice");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        assert_eq!(initial_balance + U256::ONE, balance);
    }

    // ----------------------- Acceptance-check failures ----------------------

    #[motsu::test]
    fn safe_mint_rejects_when_receiver_returns_wrong_selector(
        contract: Contract<Erc721>,
        bad: Contract<BadSelectorReceiver721>,
        alice: Address,
    ) {
        let token_id = uint!(42_U256);
        let err = contract
            .sender(alice)
            ._safe_mint(bad.address(), token_id, &vec![].into())
            .motsu_expect_err(
                "receiver returning wrong selector must be rejected",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver { receiver }) if receiver == bad.address()
        ));
        // Ensure token not minted
        let balance =
            contract.sender(alice).balance_of(bad.address()).motsu_unwrap();
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn safe_mint_bubbles_revert_reason_from_receiver(
        contract: Contract<Erc721>,
        reverting: Contract<RevertingReceiver721>,
        alice: Address,
    ) {
        let token_id = uint!(43_U256);
        let err = contract
            .sender(alice)
            ._safe_mint(reverting.address(), token_id, &vec![].into())
            .motsu_expect_err("receiver reverting should bubble reason");

        assert!(matches!(
            err,
            Error::InvalidReceiverWithReason(super::InvalidReceiverWithReason { reason }) if reason == "Receiver rejected"
        ));
        // Ensure token not minted
        let balance = contract
            .sender(alice)
            .balance_of(reverting.address())
            .motsu_unwrap();
        assert_eq!(U256::ZERO, balance);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: un-ignore when https://github.com/OpenZeppelin/stylus-test-helpers/issues/118 is fixed"]
    fn safe_mint_rejects_on_empty_revert_reason(
        contract: Contract<Erc721>,
        empty: Contract<EmptyReasonReceiver721>,
        alice: Address,
    ) {
        let token_id = uint!(44_U256);
        let err = contract
            .sender(alice)
            ._safe_mint(empty.address(), token_id, &vec![].into())
            .motsu_expect_err("empty revert must map to InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver { receiver }) if receiver == empty.address()
        ));
    }

    #[motsu::test]
    fn safe_transfer_rejects_when_receiver_returns_wrong_selector(
        contract: Contract<Erc721>,
        bad: Contract<BadSelectorReceiver721>,
        alice: Address,
    ) {
        let token_id = uint!(45_U256);
        // Mint to alice
        contract.sender(alice)._mint(alice, token_id).motsu_unwrap();

        let err = contract
            .sender(alice)
            .safe_transfer_from(alice, bad.address(), token_id)
            .motsu_expect_err("wrong selector should be rejected in transfer");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver { receiver }) if receiver == bad.address()
        ));
        // State unchanged
        let owner = contract.sender(alice).owner_of(token_id).motsu_unwrap();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfer_bubbles_revert_reason_from_receiver(
        contract: Contract<Erc721>,
        reverting: Contract<RevertingReceiver721>,
        alice: Address,
    ) {
        let token_id = uint!(46_U256);
        // Mint to alice
        contract.sender(alice)._mint(alice, token_id).motsu_unwrap();

        let err = contract
            .sender(alice)
            .safe_transfer_from(alice, reverting.address(), token_id)
            .motsu_expect_err("revert reason should bubble in transfer");

        assert!(matches!(
            err,
            Error::InvalidReceiverWithReason(super::InvalidReceiverWithReason { reason }) if reason == "Receiver rejected"
        ));
        // State unchanged
        let owner = contract.sender(alice).owner_of(token_id).motsu_unwrap();
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_minting_token_id_twice(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint the token a first time");
        let err = contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect_err("should not mint a token with `TOKEN_ID` twice");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn error_when_minting_token_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            ._mint(invalid_receiver, TOKEN_ID)
            .motsu_expect_err("should not mint a token for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn safe_mints(contract: Contract<Erc721>, alice: Address) {
        let initial_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        contract
            .sender(alice)
            ._safe_mint(alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect("should mint a token for Alice");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        assert_eq!(initial_balance + U256::ONE, balance);
    }

    #[motsu::test]
    fn error_when_safe_mint_token_id_twice(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint the token a first time");

        let err = contract
            .sender(alice)
            ._safe_mint(alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect_err("should not mint a token with `TOKEN_ID` twice");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn error_when_safe_mint_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            ._safe_mint(invalid_receiver, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect_err("should not mint a token for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn transfers_from(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");
        contract
            .sender(alice)
            .transfer_from(alice, bob, TOKEN_ID)
            .motsu_expect("should transfer a token from Alice to Bob");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn transfers_from_approved_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        contract
            .sender(bob)
            .approve(alice, TOKEN_ID)
            .motsu_expect("should approve Bob's token for Alice");
        contract
            .sender(alice)
            .transfer_from(bob, alice, TOKEN_ID)
            .motsu_expect("should transfer Bob's token to Alice");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_from_approved_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve all Bob's tokens for Alice");

        let approved_for_all =
            contract.sender(alice).is_approved_for_all(bob, alice);
        assert!(approved_for_all);

        contract
            .sender(alice)
            .transfer_from(bob, alice, TOKEN_ID)
            .motsu_expect("should transfer Bob's token to Alice");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_to_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            .transfer_from(alice, invalid_receiver, TOKEN_ID)
            .motsu_expect_err(
                "should not transfer the token to invalid receiver",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_from_incorrect_owner(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            .transfer_from(dave, bob, TOKEN_ID)
            .motsu_expect_err(
                "should not transfer the token from incorrect owner",
            );
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == dave && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .motsu_expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_with_insufficient_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        let err = contract
            .sender(alice)
            .transfer_from(bob, alice, TOKEN_ID)
            .motsu_expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                    operator,
                    token_id: t_id,
            }) if operator == alice && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            .transfer_from(alice, bob, TOKEN_ID)
            .motsu_expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                    token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn safe_transfers_from(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        contract
            .sender(alice)
            .safe_transfer_from(alice, bob, TOKEN_ID)
            .motsu_expect("should transfer a token from Alice to Bob");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");

        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        contract.sender(alice).token_approvals.setter(TOKEN_ID).set(alice);
        contract
            .sender(alice)
            .safe_transfer_from(bob, alice, TOKEN_ID)
            .motsu_expect("should transfer Bob's token to Alice");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve all Bob's tokens for Alice");

        let approved_for_all =
            contract.sender(alice).is_approved_for_all(bob, alice);
        assert!(approved_for_all);

        contract
            .sender(alice)
            .safe_transfer_from(bob, alice, TOKEN_ID)
            .motsu_expect("should transfer Bob's token to Alice");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_to_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            .safe_transfer_from(alice, invalid_receiver, TOKEN_ID)
            .motsu_expect_err(
                "should not transfer the token to invalid receiver",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_from_incorrect_owner(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            .safe_transfer_from(dave, bob, TOKEN_ID)
            .motsu_expect_err(
                "should not transfer the token from incorrect owner",
            );
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                owner,
                sender,
                token_id: t_id
            }) if sender == dave && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .motsu_expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_with_insufficient_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        let err = contract
            .sender(alice)
            .safe_transfer_from(bob, alice, TOKEN_ID)
            .motsu_expect_err("should not transfer unapproved token");
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
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            .safe_transfer_from(alice, bob, TOKEN_ID)
            .motsu_expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn safe_transfers_from_with_data(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        contract
            .sender(alice)
            .safe_transfer_from_with_data(
                alice,
                bob,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should transfer a token from Alice to Bob");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");

        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn safe_transfers_from_with_data_approved_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        contract
            .sender(bob)
            .approve(alice, TOKEN_ID)
            .motsu_expect("should approve Bob's token for Alice");
        contract
            .sender(alice)
            .safe_transfer_from_with_data(
                bob,
                alice,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should transfer Bob's token to Alice");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_from_with_data_approved_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve all Bob's tokens for Alice");

        let approved_for_all =
            contract.sender(alice).is_approved_for_all(bob, alice);
        assert!(approved_for_all);

        contract
            .sender(alice)
            .safe_transfer_from_with_data(
                bob,
                alice,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect("should transfer Bob's token to Alice");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_to_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            .safe_transfer_from_with_data(
                alice,
                invalid_receiver,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer the token to invalid receiver",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_from_incorrect_owner(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            .safe_transfer_from_with_data(
                dave,
                bob,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer the token from incorrect owner",
            );
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == dave && t_id == TOKEN_ID && owner == alice

        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .motsu_expect("should return the owner of the token");
        //
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_with_insufficient_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        let err = contract
            .sender(alice)
            .safe_transfer_from_with_data(
                bob,
                alice,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err("should not transfer unapproved token");
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
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            .safe_transfer_from_with_data(
                alice,
                bob,
                TOKEN_ID,
                vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_approve_for_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            .approve(bob, TOKEN_ID)
            .motsu_expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_by_invalid_approver(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint a token");

        let err = contract
            .sender(alice)
            .approve(dave, TOKEN_ID)
            .motsu_expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::InvalidApprover(ERC721InvalidApprover {
                approver
            }) if approver == alice
        ));
    }

    #[motsu::test]
    fn error_when_approval_for_all_for_invalid_operator(
        contract: Contract<Erc721>,
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
            Error::InvalidOperator(ERC721InvalidOperator {
                operator
            }) if operator == invalid_operator
        ));
    }

    #[motsu::test]
    fn error_when_get_approved_of_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let err =
            contract.sender(alice).get_approved(TOKEN_ID).motsu_expect_err(
                "should not return approved for a non-existent token",
            );

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn owner_of_works(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint a token");

        let owner = contract.sender(alice)._owner_of(TOKEN_ID);
        assert_eq!(bob, owner);
    }

    #[motsu::test]
    fn owner_of_nonexistent_token(contract: Contract<Erc721>, alice: Address) {
        let owner = contract.sender(alice)._owner_of(TOKEN_ID);
        assert_eq!(Address::ZERO, owner);
    }

    #[motsu::test]
    fn _get_approved_returns_zero_address_for_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let approved = contract.sender(alice)._get_approved(TOKEN_ID);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn get_approved_token_without_approval(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        let approved =
            contract.sender(alice).get_approved(TOKEN_ID).motsu_unwrap();
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn get_approved_token_with_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        contract
            .sender(alice)
            .approve(bob, TOKEN_ID)
            .motsu_expect("should approve Bob for operations on token");

        let approved =
            contract.sender(alice).get_approved(TOKEN_ID).motsu_unwrap();
        assert_eq!(bob, approved);
    }

    #[motsu::test]
    fn get_approved_token_with_approval_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        contract.sender(alice).set_approval_for_all(bob, true).motsu_expect(
            "should approve Bob for operations on all Alice's tokens",
        );

        let approved =
            contract.sender(alice).get_approved(TOKEN_ID).motsu_unwrap();
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn is_authorized_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let authorized =
            contract.sender(alice)._is_authorized(alice, bob, TOKEN_ID);
        assert!(!authorized);
    }

    #[motsu::test]
    fn is_authorized_token_owner(contract: Contract<Erc721>, alice: Address) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");

        let authorized =
            contract.sender(alice)._is_authorized(alice, alice, TOKEN_ID);
        assert!(authorized);
    }

    #[motsu::test]
    fn is_authorized_without_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");

        let authorized =
            contract.sender(alice)._is_authorized(alice, bob, TOKEN_ID);
        assert!(!authorized);
    }

    #[motsu::test]
    fn is_authorized_with_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        contract
            .sender(alice)
            .approve(bob, TOKEN_ID)
            .motsu_expect("should approve Bob for operations on token");

        let authorized =
            contract.sender(alice)._is_authorized(alice, bob, TOKEN_ID);
        assert!(authorized);
    }

    #[motsu::test]
    fn is_authorized_with_approval_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        contract.sender(alice).set_approval_for_all(bob, true).motsu_expect(
            "should approve Bob for operations on all Alice's tokens",
        );

        let authorized =
            contract.sender(alice)._is_authorized(alice, bob, TOKEN_ID);
        assert!(authorized);
    }

    #[motsu::test]
    fn check_authorized_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            ._check_authorized(Address::ZERO, alice, TOKEN_ID)
            .motsu_expect_err("should not pass for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn check_authorized_token_owner(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");

        let result =
            contract.sender(alice)._check_authorized(alice, alice, TOKEN_ID);

        assert!(result.is_ok());
    }

    #[motsu::test]
    fn check_authorized_without_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");

        let err = contract
            .sender(alice)
            ._check_authorized(alice, bob, TOKEN_ID)
            .motsu_expect_err("should not pass without approval");

        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id
            }) if operator == bob && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn check_authorized_with_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        contract
            .sender(alice)
            .approve(bob, TOKEN_ID)
            .motsu_expect("should approve Bob for operations on token");

        let result =
            contract.sender(alice)._check_authorized(alice, bob, TOKEN_ID);
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn check_authorized_with_approval_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token");
        contract.sender(alice).set_approval_for_all(bob, true).motsu_expect(
            "should approve Bob for operations on all Alice's tokens",
        );

        let result =
            contract.sender(alice)._check_authorized(alice, bob, TOKEN_ID);
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn burns(contract: Contract<Erc721>, alice: Address) {
        let one = U256::ONE;

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token for Alice");

        let initial_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        let result = contract.sender(alice)._burn(TOKEN_ID);
        let balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        let err = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect_err("should return Error::NonexistentToken");

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
    fn error_when_get_approved_of_previous_approval_burned(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token for Alice");
        contract
            .sender(alice)
            .approve(bob, TOKEN_ID)
            .motsu_expect("should approve a token for Bob");

        contract
            .sender(alice)
            ._burn(TOKEN_ID)
            .motsu_expect("should burn previously minted token");

        let err = contract
            .sender(alice)
            .get_approved(TOKEN_ID)
            .motsu_expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_burn_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            ._burn(TOKEN_ID)
            .motsu_expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn transfers(contract: Contract<Erc721>, alice: Address, bob: Address) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");
        contract
            .sender(alice)
            ._transfer(alice, bob, TOKEN_ID)
            .motsu_expect("should transfer a token from Alice to Bob");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn transfers_approved_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        contract
            .sender(bob)
            .approve(alice, TOKEN_ID)
            .motsu_expect("should approve Bob's token for Alice");
        contract
            .sender(alice)
            ._transfer(bob, alice, TOKEN_ID)
            .motsu_expect("should transfer Bob's token to Alice");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_approved_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve all Bob's tokens for Alice");

        let approved_for_all =
            contract.sender(alice).is_approved_for_all(bob, alice);
        assert!(approved_for_all);

        contract
            .sender(alice)
            ._transfer(bob, alice, TOKEN_ID)
            .motsu_expect("should transfer Bob's token to Alice");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_to_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            ._transfer(alice, invalid_receiver, TOKEN_ID)
            .motsu_expect_err("should not transfer to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_from_incorrect_owner(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            ._transfer(dave, bob, TOKEN_ID)
            .motsu_expect_err("should not transfer from incorrect owner");

        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == dave && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .motsu_expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            ._transfer(alice, bob, TOKEN_ID)
            .motsu_expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn safe_transfers_internal(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        contract
            .sender(alice)
            ._safe_transfer(alice, bob, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect("should transfer a token from Alice to Bob");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");

        assert_eq!(owner, bob);
    }

    #[motsu::test]
    fn safe_transfers_internal_approved_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");
        contract
            .sender(bob)
            .approve(alice, TOKEN_ID)
            .motsu_expect("should approve Bob's token for Alice");
        contract
            .sender(alice)
            ._safe_transfer(bob, alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect("should transfer Bob's token to Alice");
        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_internal_approved_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint token to Bob");

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve all Bob's tokens for Alice");

        let approved_for_all =
            contract.sender(alice).is_approved_for_all(bob, alice);
        assert!(approved_for_all);

        contract
            .sender(alice)
            ._safe_transfer(bob, alice, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect("should transfer Bob's token to Alice");

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_to_invalid_receiver(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_receiver = Address::ZERO;

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            ._safe_transfer(
                alice,
                invalid_receiver,
                TOKEN_ID,
                &vec![0, 1, 2, 3].into(),
            )
            .motsu_expect_err(
                "should not transfer the token to invalid receiver",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .motsu_expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_from_incorrect_owner(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .motsu_expect("should mint a token to Alice");

        let err = contract
            .sender(alice)
            ._safe_transfer(dave, bob, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect_err(
                "should not transfer the token from incorrect owner",
            );
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == dave && t_id == TOKEN_ID && owner == alice
        ));

        // NOTE: We can't check this here, but we cover this in our e2e tests.
        // let owner = contract
        // .owner_of(TOKEN_ID)
        // .motsu_expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            ._safe_transfer(alice, bob, TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_expect_err("should not transfer a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_approve_internal_for_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            ._approve(bob, TOKEN_ID, alice, false)
            .motsu_expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_internal_by_invalid_approver(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint a token");

        let err = contract
            .sender(alice)
            ._approve(dave, TOKEN_ID, alice, false)
            .motsu_expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::InvalidApprover(ERC721InvalidApprover {
                approver
            }) if approver == alice
        ));
    }

    #[motsu::test]
    fn error_when_approval_for_all_internal_for_invalid_operator(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let invalid_operator = Address::ZERO;

        let err = contract
            .sender(alice)
            ._set_approval_for_all(alice, invalid_operator, true)
            .motsu_expect_err(
                "should not approve for all for invalid operator",
            );

        assert!(matches!(
            err,
            Error::InvalidOperator(ERC721InvalidOperator {
                operator
            }) if operator == invalid_operator
        ));
    }

    #[motsu::test]
    fn require_owned_works(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .motsu_expect("should mint a token");

        let owner = contract
            .sender(alice)
            ._require_owned(TOKEN_ID)
            .motsu_expect("should return the owner of the token");

        assert_eq!(bob, owner);
    }

    #[motsu::test]
    fn error_when_require_owned_for_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            ._require_owned(TOKEN_ID)
            .motsu_expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if TOKEN_ID == t_id
        ));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc721 as IErc721>::interface_id();
        let expected: B32 = fixed_bytes!("80ac58cd");
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc721>, alice: Address) {
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc721 as IErc721>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc721 as IErc165>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }

    sol_storage! {
        pub struct Erc721ReceiverMock {
            uint256 _received_token_id;
        }
    }

    #[public]
    impl Erc721ReceiverMock {
        #[selector(name = "onERC721Received")]
        fn on_erc721_received(
            &mut self,
            _operator: Address,
            _from: Address,
            token_id: U256,
            _data: Bytes,
        ) -> B32 {
            self._received_token_id.set(token_id);
            fixed_bytes!("150b7a02")
        }

        fn received_token_id(&self) -> U256 {
            self._received_token_id.get()
        }
    }

    unsafe impl TopLevelStorage for Erc721ReceiverMock {}

    #[motsu::test]
    fn on_erc721_received(
        erc721: Contract<Erc721>,
        receiver: Contract<Erc721ReceiverMock>,
        alice: Address,
    ) {
        erc721
            .sender(alice)
            ._safe_mint(receiver.address(), TOKEN_ID, &vec![0, 1, 2, 3].into())
            .motsu_unwrap();

        let received_token_id = receiver.sender(alice).received_token_id();

        assert_eq!(received_token_id, TOKEN_ID);
    }
}
