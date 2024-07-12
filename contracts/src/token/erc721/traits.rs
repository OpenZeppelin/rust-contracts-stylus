use alloy_primitives::{Address, U256};
use stylus_proc::external;
use stylus_sdk::{abi::Bytes, msg, prelude::TopLevelStorage};

use crate::token::erc721::{base::*, Error};

/// Required interface of an [`Erc721`] compliant contract.
pub trait IErc721 {
    /// Returns the number of tokens in `owner`'s account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    ///
    /// # Errors
    ///
    /// If owner address is `Address::ZERO`, then the error
    /// [`Error::InvalidOwner`] is returned.
    fn balance_of(&self, owner: Address) -> Result<U256, Error>;

    /// Returns the owner of the `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements
    ///
    /// * `token_id` must exist.
    fn owner_of(&self, token_id: U256) -> Result<Address, Error>;

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
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If the previous owner is not `from`, then the error
    /// [`Error::IncorrectOwner`] is returned.
    /// If the caller does not have the right to approve, then the error
    /// [`Error::InsufficientApproval`] is returned.
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    /// If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * The `token_id` token must exist and be owned by `from`.
    /// * If the caller is not `from`, it must have been allowed to move this
    ///   token by either [`Self::approve`] or [`Self::set_approval_for_all`].
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon a
    ///   `safe_transfer`.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn safe_transfer_from(
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error>;

    /// Safely transfers `token_id` token from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Self::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If the previous owner is not `from`, then the error
    /// [`Error::IncorrectOwner`] is returned.
    /// If the caller does not have the right to approve, then the error
    /// [`Error::InsufficientApproval`] is returned.
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    /// If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * The `token_id` token must exist and be owned by `from`.
    /// * If the caller is not `from`, it must be approved to move this token by
    ///   either [`Self::_approve`] or [`Self::set_approval_for_all`].
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon a
    ///   `safe_transfer`.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn safe_transfer_from_with_data(
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error>;

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
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If the previous owner is not `from`, then the error
    /// [`Error::IncorrectOwner`] is returned.
    /// If the caller does not have the right to approve, then the error
    /// [`Error::InsufficientApproval`] is returned.
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * The `token_id` token must be owned by `from`.
    /// * If the caller is not `from`, it must be approved to move this token by
    ///   either [`Self::approve`] or [`Self::set_approval_for_all`].
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn transfer_from(
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error>;

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
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    /// If `auth` (param of [`Self::_approve`]) does not have a right to
    /// approve this token, then the error
    /// [`Error::InvalidApprover`] is returned.
    ///
    /// # Requirements:
    ///
    /// * The caller must own the token or be an approved operator.
    /// * `token_id` must exist.
    ///
    /// # Events
    ///
    /// Emits an [`Approval`] event.
    fn approve(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error>;

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
    ) -> Result<(), Error>;

    /// Returns the account approved for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    fn get_approved(&self, token_id: U256) -> Result<Address, Error>;

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
