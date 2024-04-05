use alloy_primitives::{fixed_bytes, Address, FixedBytes, U128, U256};
use derive_more::From;
use stylus_sdk::{
    abi::Bytes, alloy_sol_types::sol, call::Call, evm, msg, prelude::*,
};

use crate::arithmetic::{AddAssignUnchecked, SubAssignUnchecked};

sol! {
    /// Emitted when the `tokenId` token is transferred from `from` to `to`.
    ///
    /// * `from` - Address from which the token will be transferred.
    /// * `to` - Address where the token will be transferred to.
    /// * `token_id` - Token id as a number.
    event Transfer(address indexed from, address indexed to, uint256 indexed token_id);

    /// Emitted when `owner` enables `approved` to manage the `tokenId` token.
    ///
    /// * `owner` - Address of the owner of the token.
    /// * `approved` - Address of the approver.
    /// * `token_id` - Token id as a number.
    event Approval(address indexed owner, address indexed approved, uint256 indexed token_id);

    /// Emitted when `owner` enables or disables (`approved`) `operator` to manage all of its assets.
    ///
    /// * `owner` - Address of the owner of the token.
    /// * `operator` - Address of an operator that will manage operations on the token.
    /// * `approved` - Whether or not permission has been granted. If true, this means `operator` will be allowed to manage `owner`'s assets.
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
}

sol! {
    /// Indicates that an address can't be an owner.
    /// For example, `address(0)` is a forbidden owner in ERC-721. Used in balance queries.
    ///
    /// * `owner` - The address deemed to be an invalid owner.
    #[derive(Debug)]
    error ERC721InvalidOwner(address owner);

    /// Indicates a `tokenId` whose `owner` is the zero address.
    ///
    /// * `token_id` - Token id as a number.
    #[derive(Debug)]
    error ERC721NonexistentToken(uint256 token_id);

    /// Indicates an error related to the ownership over a particular token. Used in transfers.
    ///
    /// * `sender` - Address whose tokens are being transferred.
    /// * `token_id` - Token id as a number.
    /// * `owner` - Address of the owner of the token.
    #[derive(Debug)]
    error ERC721IncorrectOwner(address sender, uint256 token_id, address owner);

    /// Indicates a failure with the token `sender`. Used in transfers.
    ///
    /// * `sender` - An address whose token is being transferred.
    #[derive(Debug)]
    error ERC721InvalidSender(address sender);

    /// Indicates a failure with the token `receiver`. Used in transfers.
    ///
    /// * `receiver` - Address that receives the token.
    #[derive(Debug)]
    error ERC721InvalidReceiver(address receiver);

    /// Indicates a failure with the `operator`’s approval. Used in transfers.
    ///
    /// * `operator` - Address that may be allowed to operate on tokens without being their owner.
    /// * `token_id` - Token id as a number.
    #[derive(Debug)]
    error ERC721InsufficientApproval(address operator, uint256 token_id);

    /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
    ///
    /// * `approver` - Address of an approver that failed to approve.
    #[derive(Debug)]
    error ERC721InvalidApprover(address approver);

    /// Indicates a failure with the `operator` to be approved. Used in approvals.
    ///
    /// * `operator` - Address that may be allowed to operate on tokens without being their owner.
    #[derive(Debug)]
    error ERC721InvalidOperator(address operator);
}

/// An ERC-721 error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug, From)]
pub enum Error {
    InvalidOwner(ERC721InvalidOwner),
    NonexistentToken(ERC721NonexistentToken),
    IncorrectOwner(ERC721IncorrectOwner),
    InvalidSender(ERC721InvalidSender),
    InvalidReceiver(ERC721InvalidReceiver),
    InsufficientApproval(ERC721InsufficientApproval),
    InvalidApprover(ERC721InvalidApprover),
    InvalidOperator(ERC721InvalidOperator),
}

sol_interface! {
    /// ERC-721 token receiver interface.
    /// Interface for any contract that wants to support safeTransfers
    /// from ERC-721 asset contracts.
    interface IERC721Receiver {
        /// Whenever an [`ERC721`] `tokenId` token is transferred to this contract via [`ERC721::safe_transfer_from`]
        /// by `operator` from `from`, this function is called.
        ///
        /// It must return its Solidity selector to confirm the token transfer.
        /// If any other value is returned or the interface is not implemented by the recipient, the transfer will be
        /// reverted.
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}

sol_storage! {
    pub struct ERC721 {
        mapping(uint256 => address) _owners;

        mapping(address => uint256) _balances;

        mapping(uint256 => address) _token_approvals;

        mapping(address => mapping(address => bool)) _operator_approvals;
    }
}

// NOTE: implementation of `TopLevelStorage` to be able refer to &mut self when
// calling other contracts and not `&mut (impl TopLevelStorage +
// BorrowMut<Self>)`. Should be fixed in future by stylus team.
unsafe impl TopLevelStorage for ERC721 {}

#[external]
impl ERC721 {
    /// Returns the number of tokens in `owner` 's account.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    ///
    /// # Errors
    ///
    /// * If owner address is `Address::ZERO`, then [`Error::InvalidOwner`] is
    ///   returned.
    pub fn balance_of(&self, owner: Address) -> Result<U256, Error> {
        if owner.is_zero() {
            return Err(ERC721InvalidOwner { owner: Address::ZERO }.into());
        }
        Ok(self._balances.get(owner))
    }

    /// Returns the owner of the `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements
    ///
    /// * `token_id` must exist.
    pub fn owner_of(&self, token_id: U256) -> Result<Address, Error> {
        self._require_owned(token_id)
    }

    /// Safely transfers `token_id` token from `from` to `to`, checking first
    /// that contract recipients are aware of the ERC-721 protocol to
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
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    /// * If previous owner is not `from` then [`Error::IncorrectOwner`] is
    ///   returned.
    /// * If caller does not have right to approve then
    ///   [`Error::InsufficientApproval`] is returned.
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    /// * If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    ///   interface id or
    /// returned with error then [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * `token_id` token must exist and be owned by `from`.
    /// * If the caller is not `from`, it must have been allowed to move this
    ///   token by either [`Self::approve`] or
    /// * [`Self::set_approval_for_all`].
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon
    /// * a safe transfer.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        // TODO: use bytes! macro later
        self.safe_transfer_from_with_data(from, to, token_id, vec![].into())
    }

    /// Safely transfers `token_id` token from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    /// * If previous owner is not `from` then [`Error::IncorrectOwner`] is
    ///   returned.
    /// * If caller does not have right to approve then
    ///   [`Error::InsufficientApproval`] is returned.
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    /// * If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    ///   interface id or
    /// returned with error then [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * `token_id` token must exist and be owned by `from`.
    /// * If the caller is not `from`, it must be approved to move this token by
    ///   either [`Self::_approve`] or [`Self::set_approval_for_all`].
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon
    /// * a safe transfer.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.transfer_from(from, to, token_id)?;
        self._check_on_erc721_received(msg::sender(), from, to, token_id, data)
    }

    /// Transfers `token_id` token from `from` to `to`.
    ///
    /// WARNING: Note that the caller is responsible to confirm that the
    /// recipient is capable of receiving ERC-721 or else they may be
    /// permanently lost. Usage of [`Self::safe_transfer_from`] prevents loss,
    /// though the caller must understand this adds an external call which
    /// potentially creates a reentrancy vulnerability.
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
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    /// * If previous owner is not `from` then [`Error::IncorrectOwner`] is
    ///   returned.
    /// * If caller does not have right to approve then
    ///   [`Error::InsufficientApproval`] is returned.
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `from` cannot be the zero address.
    /// * `to` cannot be the zero address.
    /// * `token_id` token must be owned by `from`.
    /// * If the caller is not `from`, it must be approved to move this token by
    ///   either [`Self::approve`] or [`Self::set_approval_for_all`].
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn transfer_from(
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

        // Setting an "auth" arguments enables the `_isAuthorized` check which
        // verifies that the token exists (from != 0). Therefore, it is
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

    /// Gives permission to `to` to transfer `token_id` token to another
    /// account. The approval is cleared when the token is transferred.
    ///
    /// Only a single account can be approved at a time, so approving the zero
    /// address clears previous approvals.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    /// * If `auth` does not have a right to approve this token then
    ///   [`Error::InvalidApprover`] is returned.
    ///
    /// # Requirements:
    ///
    /// - The caller must own the token or be an approved operator.
    /// - `token_id` must exist.
    ///
    /// # Events
    ///
    /// Emits an [`Approval`] event.
    pub fn approve(
        &mut self,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        self._approve(to, token_id, msg::sender(), true)
    }

    /// Approve or remove `operator` as an operator for the caller.
    /// Operators can call [`Self::transfer_from`] or
    /// [`Self::safe_transfer_from`] for any token owned by the caller.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account add to the set of authorized operators.
    /// * `approved` - Flag that that set approval or disapproval for the
    ///   operator.
    ///
    /// # Errors
    ///
    /// * If `operator` is `Address::ZERO` then [`Error::InvalidOperator`] is
    ///   returned.
    ///
    /// # Requirements:
    ///
    /// * The `operator` cannot be the address zero.
    ///
    /// # Events
    ///
    /// Emits an [`ApprovalForAll`] event.
    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        self._set_approval_for_all(msg::sender(), operator, approved)
    }

    /// Returns the account approved for `token_id` token.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    pub fn get_approved(&self, token_id: U256) -> Result<Address, Error> {
        self._require_owned(token_id)?;
        Ok(self._get_approved_inner(token_id))
    }

    /// Returns if the `operator` is allowed to manage all the assets of
    /// `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account of the token's owner.
    /// * `operator` - Account to add to the set of authorized operators.
    ///
    /// # Events
    ///
    /// Emits an [`set_approval_for_all`] event.
    pub fn is_approved_for_all(
        &self,
        owner: Address,
        operator: Address,
    ) -> bool {
        self._operator_approvals.get(owner).get(operator)
    }
}

impl ERC721 {
    /// Returns the owner of the `token_id`. Does NOT revert if token doesn't
    /// exist.
    ///
    /// IMPORTANT: Any overrides to this function that add ownership of tokens
    /// not tracked by the core ERC-721 logic MUST be matched with the use
    /// of [`Self::_increase_balance`] to keep balances consistent with
    /// ownership. The invariant to preserve is that for any address `a` the
    /// value returned by `balance_of(a)` must be equal to the number of
    /// tokens such that `owner_of_inner(token_id)` is `a`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    pub fn _owner_of_inner(&self, token_id: U256) -> Address {
        self._owners.get(token_id)
    }

    /// Returns the approved address for `token_id`. Returns 0 if `token_id` is
    /// not minted.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    pub fn _get_approved_inner(&self, token_id: U256) -> Address {
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
    pub fn _is_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> bool {
        !spender.is_zero()
            && (owner == spender
                || self.is_approved_for_all(owner, spender)
                || self._get_approved_inner(token_id) == spender)
    }

    /// Checks if `spender` can operate on `token_id`, assuming the provided
    /// `owner` is the actual owner. Reverts if `spender` does not have
    /// approval from the provided `owner` for the given token or for all its
    /// assets the `spender` for the specific `token_id`.
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
    ///
    /// # Errors
    ///
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    /// * If spender does not have right to approve then
    ///   [`Error::InsufficientApproval`] is returned.
    pub fn _check_authorized(
        &self,
        owner: Address,
        spender: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if !self._is_authorized(owner, spender, token_id) {
            return if owner.is_zero() {
                Err(ERC721NonexistentToken { token_id }.into())
            } else {
                Err(ERC721InsufficientApproval { operator: spender, token_id }
                    .into())
            };
        }
        Ok(())
    }

    /// Unsafe write access to the balances, used by extensions that "mint"
    /// tokens using an [`Self::owner_of`] override.
    ///
    /// NOTE: the value is limited to type(uint128).max. This protect against
    /// _balance overflow. It is unrealistic that a uint256 would ever
    /// overflow from increments when these increments are bounded to uint128
    /// values.
    ///
    /// WARNING: Increasing an account's balance using this function tends to be
    /// paired with an override of the [`Self::_owner_of_inner`] function to
    /// resolve the ownership of the corresponding tokens so that balances and
    /// ownership remain consistent with one another.
    ///
    /// # Arguments    
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Account to increase balance.
    /// * `value` - The number of tokens to increase balance.
    // TODO: right now this function is pointless since it is not used. But once
    // we will be able to override internal functions it will make a difference
    pub fn _increase_balance(&mut self, account: Address, value: U128) {
        self._balances.setter(account).add_assign_unchecked(U256::from(value));
    }

    /// Transfers `token_id` from its current owner to `to`, or alternatively
    /// mints (or burns) if the current owner (or `to`) is the zero address.
    /// Returns the owner of the `token_id` before the update.
    ///
    /// The `auth` argument is optional. If the value passed is non 0, then this
    /// function will check that `auth` is either the owner of the token, or
    /// approved to operate on the token (by the owner).
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
    /// * If token does not exist and `auth` is not `Address::ZERO` then
    /// [`Error::NonexistentToken`] is returned.
    /// * If `auth` is not `Address::ZERO` and `auth` does not have a right to
    ///   approve this token
    /// then [`Error::InsufficientApproval`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let from = self._owner_of_inner(token_id);

        // Perform (optional) operator check
        if !auth.is_zero() {
            self._check_authorized(from, auth, token_id)?;
        }

        // Execute the update
        if !from.is_zero() {
            // Clear approval. No need to re-authorize or emit the Approval
            // event
            self._approve(Address::ZERO, token_id, Address::ZERO, false)?;
            self._balances.setter(from).sub_assign_unchecked(U256::from(1));
        }

        if !to.is_zero() {
            self._balances.setter(to).add_assign_unchecked(U256::from(1));
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
    /// * If `token_id` already exist then [`Error::InvalidSender`] is returned.
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * `to` cannot be the zero address.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
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

    /// Mints `tokenId`, transfers it to `to` and checks for `to` acceptance.
    /// An additional `data` parameter is forwarded to
    /// [`IERC721Receiver::on_erc_721_received`] to contract recipients.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * If `token_id` already exist then [`Error::InvalidSender`] is returned.
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    /// * If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    ///   interface id or returned with error then [`Error::InvalidReceiver`] is
    ///   returned.
    ///
    /// # Requirements:
    ///
    /// * `tokenId` must not exist.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon a safe
    ///   transfer.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _safe_mint(
        &mut self,
        to: Address,
        token_id: U256,
        data: Bytes,
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
    /// The approval is cleared when the token is burned.
    /// This is an internal function that does not check if the sender is
    /// authorized to operate on the token.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _burn(&mut self, token_id: U256) -> Result<(), Error> {
        let previous_owner =
            self._update(Address::ZERO, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            Err(ERC721NonexistentToken { token_id }.into())
        } else {
            Ok(())
        }
    }

    /// Transfers `token_id` from `from` to `to`.
    /// As opposed to [`transferFrom`], this imposes no restrictions on
    /// msg.sender.
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
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    /// * If `token_id` does not exist then [`Error::ERC721NonexistentToken`] is
    ///   returned.
    /// * If previous owner is not `from` then [`Error::IncorrectOwner`] is
    ///   returned.
    ///
    /// # Requirements:
    ///
    /// * `to` cannot be the zero address.
    /// * `token_id` token must be owned by `from`.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
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
            Err(ERC721NonexistentToken { token_id }.into())
        } else if previous_owner != from {
            Err(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            }
            .into())
        } else {
            Ok(())
        }
    }

    /// Safely transfers `tokenId` token from `from` to `to`, checking that
    /// contract recipients are aware of the ERC-721 standard to prevent
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
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// * If `to` is `Address::ZERO` then [`Error::InvalidReceiver`] is
    ///   returned.
    /// * If `token_id` does not exist then [`Error::ERC721NonexistentToken`] is
    ///   returned.
    /// * If previous owner is not `from` then [`Error::IncorrectOwner`] is
    ///   returned.
    ///
    /// # Requirements:
    ///
    /// * `tokenId` token must exist and be owned by `from`.
    /// * `to` cannot be the zero address.
    /// * `from` cannot be the zero address.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon a safe
    ///   transfer.
    ///
    /// # Events
    /// Emits a [`Transfer`] event.
    pub fn _safe_transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self._transfer(from, to, token_id)?;
        Self::_check_on_erc721_received(
            self,
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )
    }

    /// Variant of `approve_inner` with an optional flag to enable or disable
    /// the [`Approval`] event. The event is not emitted in the context of
    /// transfers.
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
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    /// * If `auth` does not have a right to approve this token then
    ///   [`Error::InvalidApprover`]
    /// is returned
    ///
    /// # Events
    /// Emits an [`Approval`] event.
    pub fn _approve(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
        emit_event: bool,
    ) -> Result<(), Error> {
        // Avoid reading the owner unless necessary
        if emit_event || !auth.is_zero() {
            let owner = self._require_owned(token_id)?;

            // We do not use _isAuthorized because single-token approvals should
            // not be able to call approve
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
    /// * `approved` - Whether permission will be granted. If true, this means
    ///   `operator` will be allowed to manage `owner`'s assets.
    ///
    /// # Errors
    ///
    /// * If `operator` is `Address::ZERO` then [`Error::InvalidOperator`] is
    ///   returned.
    ///
    /// # Requirements:
    ///
    /// * `operator` can't be the address zero.
    ///
    /// # Events
    ///
    /// Emits an [`ApprovalForAll`] event.
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
    /// [`Self::_owner_of_inner`].
    ///
    /// # Errors
    ///
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    pub fn _require_owned(&self, token_id: U256) -> Result<Address, Error> {
        let owner = self._owner_of_inner(token_id);
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
    /// The acceptance call is not executed and treated as a no-op if the target
    /// address doesn't contain code (i.e. an EOA). Otherwise, the recipient
    /// must implement [`IERC721Receiver::on_erc_721_received`] and return the
    /// acceptance magic value to accept the transfer.
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
    /// * If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    ///   interface id or returned with error then [`Error::InvalidReceiver`] is
    ///   returned.
    pub fn _check_on_erc721_received(
        &mut self,
        operator: Address,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        const IERC721RECEIVER_INTERFACE_ID: FixedBytes<4> =
            fixed_bytes!("150b7a02");

        if to.has_code() {
            let call = Call::new_in(self);
            return match IERC721Receiver::new(to).on_erc_721_received(
                call,
                operator,
                from,
                token_id,
                data.to_vec(),
            ) {
                Ok(result) => {
                    if result != IERC721RECEIVER_INTERFACE_ID {
                        Err(ERC721InvalidReceiver { receiver: to }.into())
                    } else {
                        Ok(())
                    }
                }
                Err(_) => Err(ERC721InvalidReceiver { receiver: to }.into()),
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::address;
    use once_cell::sync::Lazy;
    use stylus_sdk::storage::StorageMap;

    use crate::erc721::*;

    // NOTE: Alice is always the sender of the message
    static ALICE: Lazy<Address> = Lazy::new(msg::sender);

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    impl Default for ERC721 {
        fn default() -> Self {
            let root = U256::ZERO;

            ERC721 {
                _owners: unsafe { StorageMap::new(root, 0) },
                _balances: unsafe { StorageMap::new(root + U256::from(32), 0) },
                _token_approvals: unsafe {
                    StorageMap::new(root + U256::from(64), 0)
                },
                _operator_approvals: unsafe {
                    StorageMap::new(root + U256::from(96), 0)
                },
            }
        }
    }

    fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        num.try_into().expect("conversion to U256")
    }

    #[grip::test]
    fn mint(contract: ERC721) {
        let token_id = random_token_id();
        contract._mint(*ALICE, token_id).expect("mint a token for Alice");
        let owner =
            contract.owner_of(token_id).expect("get the owner of the token");
        assert_eq!(owner, *ALICE);

        let balance =
            contract.balance_of(*ALICE).expect("get the balance of Alice");
        let one = U256::from(1);
        assert!(balance >= one);
    }

    #[grip::test]
    fn error_when_reusing_token_id(contract: ERC721) {
        let token_id = random_token_id();
        contract._mint(*ALICE, token_id).expect("mint the token a first time");
        let err = contract
            ._mint(*ALICE, token_id)
            .expect_err("should not mint a token id twice");
        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[grip::test]
    fn transfer(contract: ERC721) {
        let token_id = random_token_id();
        contract._mint(*ALICE, token_id).expect("mint a token to Alice");
        contract
            .transfer_from(*ALICE, BOB, token_id)
            .expect("transfer from Alice to Bob");
        let owner =
            contract.owner_of(token_id).expect("get the owner of the token");
        assert_eq!(owner, BOB);
    }

    #[grip::test]
    fn error_when_transfer_nonexistent_token(contract: ERC721) {
        let token_id = random_token_id();
        let err = contract
            .transfer_from(*ALICE, BOB, token_id)
            .expect_err("should not transfer a non existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                    token_id: t_id,
            }) if t_id == token_id
        ));
    }

    #[grip::test]
    fn approve_token_transfer(contract: ERC721) {
        let token_id = random_token_id();
        contract._mint(*ALICE, token_id).expect("mint token");
        contract
            .approve(BOB, token_id)
            .expect("approve bob for operations on token");
        assert_eq!(contract._token_approvals.get(token_id), BOB);
    }

    #[grip::test]
    fn transfer_approved_token(contract: ERC721) {
        let token_id = random_token_id();
        contract._mint(BOB, token_id).expect("mint token to Bob");
        contract._token_approvals.setter(token_id).set(*ALICE);
        contract
            .transfer_from(BOB, *ALICE, token_id)
            .expect("transfer Bob's token to Alice");
        let owner =
            contract.owner_of(token_id).expect("get the owner of the token");
        assert_eq!(owner, *ALICE);
    }

    #[grip::test]
    fn error_when_transfer_unapproved_token(contract: ERC721) {
        let token_id = random_token_id();
        contract._mint(BOB, token_id).expect("mint token to Bob");
        let err = contract
            .transfer_from(BOB, *ALICE, token_id)
            .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                    operator,
                    token_id: t_id,
            }) if operator == *ALICE && t_id == token_id
        ));
    }

    // TODO: add set_approval_for_all test

    // TODO: add mock test for on_erc721_received
}
