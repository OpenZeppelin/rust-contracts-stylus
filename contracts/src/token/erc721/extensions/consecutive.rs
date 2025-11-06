//! Implementation of the ERC-2309 "Consecutive Transfer Extension" as defined
//! in [ERC-2309].
//!
//! This extension allows the minting large batches of tokens, during
//! contract construction only. For upgradeable contracts, this implies that
//! batch minting is only available during proxy deployment, and not in
//! subsequent upgrades. These batches are limited to 5000 tokens at a time by
//! default to accommodate off-chain indexers.
//!
//! Using this extension removes the ability to mint single tokens during
//! contract construction. This ability is regained after construction. During
//! construction, only batch minting is allowed.
//!
//! Fields `first_consecutive_id` (used to offset first token id) and
//! `max_batch_size` (used to restrict maximum batch size) can be assigned
//! during construction.
//!
//! [ERC-2309]: https://eips.ethereum.org/EIPS/eip-2309

use alloc::{vec, vec::Vec};

use alloy_primitives::{
    aliases::{B32, U96},
    uint, Address, U256,
};
use stylus_sdk::{abi::Bytes, call::MethodError, evm, msg, prelude::*};

use crate::{
    token::erc721::{
        self, Approval, ERC721IncorrectOwner, ERC721InsufficientApproval,
        ERC721InvalidApprover, ERC721InvalidOperator, ERC721InvalidOwner,
        ERC721InvalidReceiver, ERC721InvalidSender, ERC721NonexistentToken,
        Erc721, IErc721, InvalidReceiverWithReason, Transfer,
    },
    utils::{
        introspection::erc165::IErc165,
        math::storage::{AddAssignUnchecked, SubAssignUnchecked},
        structs::{
            bitmap::BitMap,
            checkpoints::{
                self, CheckpointUnorderedInsertion, Size, Trace, S160,
            },
        },
    },
};

type StorageU96 = <S160 as Size>::KeyStorage;

pub use sol::*;
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the tokens from `from_token_id` to `to_token_id` are transferred from `from_address` to `to_address`.
        ///
        /// * `from_token_id` - First token being transferred.
        /// * `to_token_id` - Last token being transferred.
        /// * `from_address` - Address from which tokens will be transferred.
        /// * `to_address` - Address where the tokens will be transferred to.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event ConsecutiveTransfer(
            uint256 indexed from_token_id,
            uint256 to_token_id,
            address indexed from_address,
            address indexed to_address
        );
    }

    sol! {
        /// Batch mint is restricted to the constructor.
        /// Any batch mint not emitting the [`Transfer`] event outside of the constructor
        /// is non ERC-721 compliant.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721ForbiddenBatchMint();

        /// Exceeds the max number of mints per batch.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721ExceededMaxBatchMint(uint256 batch_size, uint256 max_batch);

        /// Individual minting is not allowed.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721ForbiddenMint();

        /// Batch burn is not supported.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC721ForbiddenBatchBurn();
    }
}

/// An [`Erc721Consecutive`] error.
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
    /// A value was attempted to be inserted into a past checkpoint.
    CheckpointUnorderedInsertion(CheckpointUnorderedInsertion),
    /// Batch mint is restricted to the constructor.
    /// Any batch mint not emitting the [`Transfer`] event outside of
    /// the constructor is non ERC-721 compliant.
    ForbiddenBatchMint(ERC721ForbiddenBatchMint),
    /// Exceeds the max amount of mints per batch.
    ExceededMaxBatchMint(ERC721ExceededMaxBatchMint),
    /// Individual minting is not allowed.
    ForbiddenMint(ERC721ForbiddenMint),
    /// Batch burn is not supported.
    ForbiddenBatchBurn(ERC721ForbiddenBatchBurn),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<erc721::Error> for Error {
    fn from(value: erc721::Error) -> Self {
        match value {
            erc721::Error::InvalidOwner(e) => Error::InvalidOwner(e),
            erc721::Error::NonexistentToken(e) => Error::NonexistentToken(e),
            erc721::Error::IncorrectOwner(e) => Error::IncorrectOwner(e),
            erc721::Error::InvalidSender(e) => Error::InvalidSender(e),
            erc721::Error::InvalidReceiver(e) => Error::InvalidReceiver(e),
            erc721::Error::InvalidReceiverWithReason(e) => {
                Error::InvalidReceiverWithReason(e)
            }
            erc721::Error::InsufficientApproval(e) => {
                Error::InsufficientApproval(e)
            }
            erc721::Error::InvalidApprover(e) => Error::InvalidApprover(e),
            erc721::Error::InvalidOperator(e) => Error::InvalidOperator(e),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl From<checkpoints::Error> for Error {
    fn from(value: checkpoints::Error) -> Self {
        match value {
            checkpoints::Error::CheckpointUnorderedInsertion(e) => {
                Error::CheckpointUnorderedInsertion(e)
            }
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc721Consecutive`] token.
#[storage]
pub struct Erc721Consecutive {
    // Must be public so that internal fields can be accessed in inheriting
    // contracts' constructors.
    /// [`Erc721`] contract.
    pub erc721: Erc721,
    /// [`Trace`] contract for sequential ownership.
    pub(crate) sequential_ownership: Trace<S160>,
    /// [`BitMap`] contract for sequential burn of tokens.
    pub(crate) sequential_burn: BitMap,
    // TODO: Remove this field once function overriding is possible. For now we
    // keep this field `pub`, since this is used to simulate overriding.
    /// Used to offset the first token id in `next_consecutive_id` calculation.
    pub first_consecutive_id: StorageU96,
    // TODO: Remove this field once function overriding is possible. For now we
    // keep this field `pub`, since this is used to simulate overriding.
    /// Maximum size of a batch of consecutive tokens. This is designed to
    /// limit stress on off-chain indexing services that have to record one
    /// entry per token, and have protections against "unreasonably large"
    /// batches of tokens.
    pub max_batch_size: StorageU96,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc721Consecutive {}

// ************** ERC-721 External **************

#[public]
impl IErc721 for Erc721Consecutive {
    type Error = Error;

    fn balance_of(&self, owner: Address) -> Result<U256, Self::Error> {
        Ok(self.erc721.balance_of(owner)?)
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
        // TODO: Once the SDK supports the conversion,
        // use alloy_primitives::bytes!("") here.
        self.safe_transfer_from_with_data(from, to, token_id, vec![].into())
    }

    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Self::Error> {
        self.transfer_from(from, to, token_id)?;
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            from,
            to,
            token_id,
            &data,
        )?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Self::Error> {
        if to.is_zero() {
            return Err(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO },
            )
            .into());
        }

        // Setting an "auth" argument enables the `_is_authorized` check which
        // verifies that the token exists (`!from.is_zero()`). Therefore, it is
        // not needed to verify that the return value is not 0 here.
        let previous_owner = self._update(to, token_id, msg::sender())?;
        if previous_owner != from {
            return Err(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            })
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
        Ok(self.erc721.set_approval_for_all(operator, approved)?)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, Self::Error> {
        self._require_owned(token_id)?;
        Ok(self.erc721._get_approved(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721.is_approved_for_all(owner, operator)
    }
}

#[public]
#[implements(IErc721<Error = Error>, IErc165)]
impl Erc721Consecutive {
    // TODO: remove once function overriding is possible, so `max_batch_size`
    // can be set that way.
    /// Constructor.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    #[constructor]
    pub fn constructor(&mut self) {
        self.max_batch_size.set(uint!(5000_U96));
    }
}

// ************** Consecutive **************

impl Erc721Consecutive {
    /// Override of [`Erc721::_owner_of`] that checks the sequential
    /// ownership structure for tokens that have been minted as part of a
    /// batch, and not yet transferred.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    pub fn _owner_of(&self, token_id: U256) -> Address {
        let owner = self.erc721._owner_of(token_id);
        // If token is owned by the core, or beyond consecutive range, return
        // base value.
        if !owner.is_zero()
            || token_id < U256::from(self._first_consecutive_id())
            || token_id > U256::from(U96::MAX)
        {
            return owner;
        }

        // Otherwise, check the token was not burned, and fetch ownership from
        // the anchors.
        if self.sequential_burn.get(token_id) {
            Address::ZERO
        } else {
            // NOTE: Bounds already checked. No need for safe cast of token_id
            self.sequential_ownership.lower_lookup(U96::from(token_id)).into()
        }
    }

    /// Mint a batch of tokens with length `batch_size` for `to`.
    /// Returns the token id of the first token minted in the batch; if
    /// `batch_size` is 0, returns the number of consecutive ids minted so
    /// far.
    ///
    /// CAUTION: Does not emit a [`Transfer`] event. This is ERC-721 compliant
    /// as long as it is done inside of the constructor, which is enforced by
    /// this function.
    ///
    /// CAUTION: Does not invoke
    /// [`erc721::IErc721Receiver::on_erc721_received`] on the receiver.
    ///
    /// # Arguments
    ///
    /// * `&self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// * [`erc721::Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`Error::ExceededMaxBatchMint`] - If `batch_size` exceeds
    ///   `max_batch_size` of the contract.
    ///
    /// # Events
    ///
    /// * [`ConsecutiveTransfer`].
    pub fn _mint_consecutive(
        &mut self,
        to: Address,
        batch_size: U96,
    ) -> Result<U96, Error> {
        let next = self._next_consecutive_id();

        // Minting a batch of size 0 is a no-op.
        if batch_size > U96::ZERO {
            if to.is_zero() {
                return Err(erc721::Error::InvalidReceiver(
                    ERC721InvalidReceiver { receiver: Address::ZERO },
                )
                .into());
            }

            if batch_size > self._max_batch_size() {
                return Err(ERC721ExceededMaxBatchMint {
                    batch_size: U256::from(batch_size),
                    max_batch: U256::from(self._max_batch_size()),
                }
                .into());
            }

            // Push an ownership checkpoint & emit event.
            let last = next + batch_size - U96::ONE;
            self.sequential_ownership.push(last, to.into())?;

            // The invariant required by this function is preserved because the
            // new sequential_ownership checkpoint is attributing
            // ownership of `batch_size` new tokens to account `to`.
            self.erc721._increase_balance(
                to,
                alloy_primitives::U128::from(batch_size),
            );

            evm::log(ConsecutiveTransfer {
                from_token_id: next.to::<U256>(),
                to_token_id: last.to::<U256>(),
                from_address: Address::ZERO,
                to_address: to,
            });
        }
        Ok(next)
    }

    /// Override of [`Erc721::_update`] that restricts normal minting to after
    /// construction.
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
    /// * [`erc721::Error::NonexistentToken`] - If token does not exist and
    ///   `auth` is not [`Address::ZERO`].
    /// * [`erc721::Error::InsufficientApproval`] - If `auth` is not
    ///   [`Address::ZERO`] and `auth` does not have a right to approve this
    ///   token.
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
        let previous_owner = self._update_base(to, token_id, auth)?;

        // if we burn
        if to.is_zero()
            // and the token_id was minted in a batch
            && token_id < U256::from(self._next_consecutive_id())
            // and the token was never marked as burnt
            && !self.sequential_burn.get(token_id)
        {
            // record burn
            self.sequential_burn.set(token_id);
        }

        Ok(previous_owner)
    }

    /// Returns the next token id to mint using [`Self::_mint_consecutive`]. It
    /// will return [`Erc721Consecutive::_first_consecutive_id`] if no
    /// consecutive token id has been minted before.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn _next_consecutive_id(&self) -> U96 {
        match self.sequential_ownership.latest_checkpoint() {
            None => self._first_consecutive_id(),
            Some((latest_id, _)) => latest_id + U96::ONE,
        }
    }

    /// Used to offset the first token id in
    /// [`Erc721Consecutive::_next_consecutive_id`].
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn _first_consecutive_id(&self) -> U96 {
        self.first_consecutive_id.get()
    }

    /// Maximum size of consecutive token's batch.
    /// This is designed to limit stress on off-chain indexing services that
    /// have to record one entry per token, and have protections against
    /// "unreasonably large" batches of tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn _max_batch_size(&self) -> U96 {
        self.max_batch_size.get()
    }
}

// ************** ERC-721 Internal **************

impl Erc721Consecutive {
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
    /// also [`Erc721::_increase_balance`].
    fn _update_base(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let from = self._owner_of(token_id);

        // Perform (optional) operator check.
        if !auth.is_zero() {
            self.erc721._check_authorized(from, auth, token_id)?;
        }

        // Execute the update.
        if !from.is_zero() {
            // Clear approval. No need to re-authorize or emit the `Approval`
            // event.
            self._approve(Address::ZERO, token_id, Address::ZERO, false)?;
            self.erc721.balances.setter(from).sub_assign_unchecked(U256::ONE);
        }

        if !to.is_zero() {
            self.erc721.balances.setter(to).add_assign_unchecked(U256::ONE);
        }

        self.erc721.owners.setter(token_id).set(to);
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
    /// * [`erc721::Error::InvalidSender`] - If `token_id` already exists.
    /// * [`erc721::Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        if to.is_zero() {
            return Err(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO },
            )
            .into());
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if !previous_owner.is_zero() {
            return Err(erc721::Error::InvalidSender(ERC721InvalidSender {
                sender: Address::ZERO,
            })
            .into());
        }
        Ok(())
    }

    /// Mints `token_id`, transfers it to `to`, and checks for `to`'s
    /// acceptance.
    ///
    /// An additional `data` parameter is forwarded to
    /// [`erc721::IErc721Receiver::on_erc721_received`] to contract recipients.
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
    /// * [`erc721::Error::InvalidSender`] - If `token_id` already exists.
    /// * [`erc721::Error::InvalidReceiver`] - If `to` is [`Address::ZERO`], or
    ///   [`erc721::IErc721Receiver::on_erc721_received`] hasn't returned its
    ///   interface id or returned with error.
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
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            Address::ZERO,
            to,
            token_id,
            data,
        )?)
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
    /// * [`erc721::Error::NonexistentToken`] - If token does not exist.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    pub fn _burn(&mut self, token_id: U256) -> Result<(), Error> {
        let previous_owner =
            self._update(Address::ZERO, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(erc721::Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
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
    /// * [`erc721::Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`erc721::Error::NonexistentToken`] - If `token_id` does not exist.
    /// * [`erc721::Error::IncorrectOwner`] - If the previous owner is not
    ///   `from`.
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
            return Err(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO },
            )
            .into());
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(erc721::Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
        } else if previous_owner != from {
            return Err(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            })
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
    /// invokes [`erc721::IErc721Receiver::on_erc721_received`] on the
    /// receiver, and can be used to e.g. implement alternative mechanisms
    /// to perform token transfer, such as signature-based.
    ///
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
    /// * [`erc721::Error::InvalidReceiver`] - If `to` is [`Address::ZERO`].
    /// * [`erc721::Error::NonexistentToken`] - If `token_id` does not exist.
    /// * [`erc721::Error::IncorrectOwner`] - If the previous owner is not
    ///   `from`.
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
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )?)
    }

    /// Approve `to` to operate on `token_id`.
    ///
    /// The `auth` argument is optional. If the value passed is non
    /// [`Address::ZERO`], then this function will check that `auth` is either
    /// the owner of the token, or approved to operate on all tokens held by
    /// this owner.
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
    /// * [`erc721::Error::NonexistentToken`] - If the token does not exist.
    /// * [`erc721::Error::InvalidApprover`] - If `auth` does not have a right
    ///   to approve this token.
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
            if !auth.is_zero()
                && owner != auth
                && !self.is_approved_for_all(owner, auth)
            {
                return Err(erc721::Error::InvalidApprover(
                    ERC721InvalidApprover { approver: auth },
                )
                .into());
            }

            if emit_event {
                evm::log(Approval { owner, approved: to, token_id });
            }
        }

        self.erc721.token_approvals.setter(token_id).set(to);
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
    /// * [`erc721::Error::NonexistentToken`] - If token does not exist.
    pub fn _require_owned(&self, token_id: U256) -> Result<Address, Error> {
        let owner = self._owner_of(token_id);
        if owner.is_zero() {
            return Err(erc721::Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
        }
        Ok(owner)
    }
}

#[public]
impl IErc165 for Erc721Consecutive {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.erc721.supports_interface(interface_id)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::*;

    use super::*;
    use crate::{
        token::erc721::receiver::tests::BadSelectorReceiver721,
        utils::introspection::erc165::IErc165,
    };

    const FIRST_CONSECUTIVE_TOKEN_ID: U96 = U96::ZERO;
    const TOKEN_ID: U256 = U256::ONE;
    const NON_CONSECUTIVE_TOKEN_ID: U256 = uint!(10001_U256);

    impl Erc721Consecutive {
        fn init(&mut self, receivers: Vec<Address>, batches: Vec<U96>) {
            self.constructor();
            for (to, batch_size) in receivers.into_iter().zip(batches) {
                self._mint_consecutive(to, batch_size)
                    .motsu_expect("should mint consecutively");
            }
        }
    }

    #[motsu::test]
    fn mints(contract: Contract<Erc721Consecutive>, alice: Address) {
        let initial_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        let init_tokens_count = uint!(10_U96);
        contract.sender(alice).init(vec![alice], vec![init_tokens_count]);

        let balance1 = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");
        assert_eq!(balance1, initial_balance + U256::from(init_tokens_count));

        // Check non-consecutive mint.
        let non_consecutive_token_id = uint!(10_U256);
        contract
            .sender(alice)
            ._mint(alice, non_consecutive_token_id)
            .motsu_expect("should mint a token for Alice");
        let owner = contract
            .sender(alice)
            .owner_of(non_consecutive_token_id)
            .motsu_expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance2 = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");

        assert_eq!(balance2, balance1 + U256::ONE);
    }

    #[motsu::test]
    fn error_when_minting_token_id_twice(
        contract: Contract<Erc721Consecutive>,
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
        contract: Contract<Erc721Consecutive>,
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
    fn error_when_to_is_zero(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            ._mint_consecutive(Address::ZERO, uint!(11_U96))
            .motsu_expect_err("should not mint consecutive");
        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver: Address::ZERO
            })
        ));
    }

    #[motsu::test]
    fn error_when_exceed_batch_size(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
    ) {
        let batch_size = contract.sender(alice)._max_batch_size() + U96::ONE;
        let err = contract
            .sender(alice)
            ._mint_consecutive(alice, batch_size)
            .motsu_expect_err("should not mint consecutive");
        assert!(matches!(
            err,
            Error::ExceededMaxBatchMint(ERC721ExceededMaxBatchMint {
                batch_size,
                max_batch
            })
            if batch_size == U256::from(batch_size) && max_batch == U256::from(contract.sender(alice)._max_batch_size())
        ));
    }

    #[motsu::test]
    fn transfers_from(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
        bob: Address,
    ) {
        // Mint batches of 1000 tokens to Alice and Bob.
        contract
            .sender(alice)
            .init(vec![alice, bob], vec![uint!(1000_U96), uint!(1000_U96)]);

        // Transfer first consecutive token from Alice to Bob.
        contract
            .sender(alice)
            .transfer_from(alice, bob, U256::from(FIRST_CONSECUTIVE_TOKEN_ID))
            .motsu_expect("should transfer a token from Alice to Bob");

        let owner = contract
            .sender(alice)
            .owner_of(U256::from(FIRST_CONSECUTIVE_TOKEN_ID))
            .motsu_expect("token should be owned");
        assert_eq!(owner, bob);

        // Check that balances changed.
        let alice_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256) - U256::ONE);
        let bob_balance = contract
            .sender(alice)
            .balance_of(bob)
            .motsu_expect("should return the balance of Bob");
        assert_eq!(bob_balance, uint!(1000_U256) + U256::ONE);

        // Check non-consecutive mint.
        contract
            .sender(alice)
            ._mint(alice, NON_CONSECUTIVE_TOKEN_ID)
            .motsu_expect("should mint a token to Alice");
        let alice_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256));

        // Check transfer of the token that wasn't minted consecutive.
        contract
            .sender(alice)
            .transfer_from(alice, bob, NON_CONSECUTIVE_TOKEN_ID)
            .motsu_expect("should transfer a token from Alice to Bob");
        let alice_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256) - U256::ONE);
    }

    #[motsu::test]
    fn burns(contract: Contract<Erc721Consecutive>, alice: Address) {
        // Mint batch of 1000 tokens to Alice.
        contract.sender(alice).init(vec![alice], vec![uint!(1000_U96)]);

        // Check consecutive token burn.
        contract
            .sender(alice)
            ._burn(U256::from(FIRST_CONSECUTIVE_TOKEN_ID))
            .motsu_expect("should burn token");

        let alice_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256) - U256::ONE);

        let err = contract
            .sender(alice)
            .owner_of(U256::from(FIRST_CONSECUTIVE_TOKEN_ID))
            .motsu_expect_err("token should not exist");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken { token_id })
            if token_id == U256::from(FIRST_CONSECUTIVE_TOKEN_ID)
        ));

        // Check non-consecutive token burn.
        let non_consecutive_token_id = uint!(2000_U256);
        contract
            .sender(alice)
            ._mint(alice, non_consecutive_token_id)
            .motsu_expect("should mint a token to Alice");
        let owner = contract
            .sender(alice)
            .owner_of(non_consecutive_token_id)
            .motsu_expect("should return owner of the token");
        assert_eq!(owner, alice);
        let alice_balance = contract
            .sender(alice)
            .balance_of(alice)
            .motsu_expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256));

        contract
            .sender(alice)
            ._burn(non_consecutive_token_id)
            .motsu_expect("should burn token");

        let err = contract
            .sender(alice)
            .owner_of(U256::from(non_consecutive_token_id))
            .motsu_expect_err("token should not exist");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken { token_id })
            if token_id == U256::from(non_consecutive_token_id)
        ));

        // After being burnt the token should not be burnt again.
        let non_existent_token = non_consecutive_token_id;
        let err = contract
            .sender(alice)
            ._burn(non_existent_token)
            .motsu_expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == non_existent_token
        ));
    }

    #[motsu::test]
    fn safe_transfer_from(
        contract: Contract<Erc721Consecutive>,
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
        contract: Contract<Erc721Consecutive>,
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
            .motsu_expect("should approve Bob's token to Alice");
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
    fn error_when_safe_transfer_from_incorrect_owner(
        contract: Contract<Erc721Consecutive>,
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
            .motsu_expect_err("should not transfer from incorrect owner");

        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == dave && t_id == TOKEN_ID && owner == alice
        ));
    }

    #[motsu::test]
    fn _safe_transfer_succeeds(
        contract: Contract<Erc721Consecutive>,
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
    fn _safe_transfer_reverts_on_nonexistent_token(
        contract: Contract<Erc721Consecutive>,
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
    fn error_when_safe_transfer_to_invalid_receiver(
        contract: Contract<Erc721Consecutive>,
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
    fn safe_transfers_from_with_data(
        contract: Contract<Erc721Consecutive>,
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
    fn safe_transfer_from_reverts_when_receiver_returns_wrong_selector(
        contract: Contract<Erc721Consecutive>,
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
    fn _safe_transfer_reverts_when_receiver_returns_wrong_selector(
        contract: Contract<Erc721Consecutive>,
        bad: Contract<BadSelectorReceiver721>,
        alice: Address,
    ) {
        let token_id = uint!(45_U256);
        // Mint to alice
        contract.sender(alice)._mint(alice, token_id).motsu_unwrap();

        let err = contract
            .sender(alice)
            ._safe_transfer(alice, bad.address(), token_id, &vec![].into())
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
    fn error_when_internal_safe_transfer_to_invalid_receiver(
        contract: Contract<Erc721Consecutive>,
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
        contract: Contract<Erc721Consecutive>,
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
    }

    #[motsu::test]
    fn safe_mint_succeeds(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
    ) {
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
    fn safe_mint_rejects_when_receiver_returns_wrong_selector(
        contract: Contract<Erc721Consecutive>,
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
    fn error_when_approve_for_nonexistent_token(
        contract: Contract<Erc721Consecutive>,
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
        contract: Contract<Erc721Consecutive>,
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
    fn approval_for_all(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
        bob: Address,
    ) {
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
    fn get_approved_token_with_approval(
        contract: Contract<Erc721Consecutive>,
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

        let approved = contract.sender(alice).get_approved(TOKEN_ID);
        assert!(matches!(approved, Ok(addr) if addr == bob));
    }

    #[motsu::test]
    fn _mint_consecutive_succeeds_for_zero_batch_size(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
    ) {
        let next = contract
            .sender(alice)
            ._mint_consecutive(alice, U96::ZERO)
            .motsu_expect("should mint consecutive tokens");
        assert_eq!(next, U96::ZERO);
    }

    #[motsu::test]
    fn error_when_get_approved_of_nonexistent_token(
        contract: Contract<Erc721Consecutive>,
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
    fn supports_interface(
        contract: Contract<Erc721Consecutive>,
        alice: Address,
    ) {
        assert!(
            contract.sender(alice).supports_interface(
                <Erc721Consecutive as IErc721>::interface_id()
            )
        );
        assert!(
            contract.sender(alice).supports_interface(
                <Erc721Consecutive as IErc165>::interface_id()
            )
        );

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }
}
