//! Implementation of the ERC-2309 "Consecutive Transfer Extension" as defined
//! in https://eips.ethereum.org/EIPS/eip-2309[ERC-2309].
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
//! Fields `_first_consecutive_id` (used to offset first token id) and
//! `_max_batch_size` (used to restrict maximum batch size) can be assigned
//! during construction with `koba` (stylus construction tooling) within
//! solidity constructor file.
//!
//! IMPORTANT: Consecutive mint of [`Erc721Consecutive`] tokens is only allowed
//! inside the contract's Solidity constructor.
//! As opposed to the Solidity implementation of Consecutive, there is no
//! restriction on the [`Erc721Consecutive::_update`] function call since it is
//! not possible to call a Rust function from the Solidity constructor.

use alloc::vec;

use alloy_primitives::{uint, Address, U256};
use alloy_sol_types::sol;
use stylus_sdk::{
    abi::Bytes,
    evm, msg,
    prelude::TopLevelStorage,
    stylus_proc::{public, sol_storage, SolidityError},
};

use crate::{
    token::{
        erc721,
        erc721::{
            Approval, ERC721IncorrectOwner, ERC721InvalidApprover,
            ERC721InvalidReceiver, ERC721InvalidSender, ERC721NonexistentToken,
            Erc721, IErc721, Transfer,
        },
    },
    utils::{
        math::storage::{AddAssignUnchecked, SubAssignUnchecked},
        structs::{
            bitmap::BitMap,
            checkpoints,
            checkpoints::{Size, Trace, S160},
        },
    },
};

type U96 = <S160 as Size>::Key;

sol_storage! {
    /// State of an [`Erc721Consecutive`] token.
    pub struct Erc721Consecutive {
        /// Erc721 contract storage.
        Erc721 erc721;
        /// Checkpoint library contract for sequential ownership.
        Trace<S160> _sequential_ownership;
        /// BitMap library contract for sequential burn of tokens.
        BitMap _sequential_burn;
        /// Used to offset the first token id in
        /// [`Erc721Consecutive::_next_consecutive_id`].
        uint96 _first_consecutive_id;
        /// Maximum size of a batch of consecutive tokens. This is designed to limit
        /// stress on off-chain indexing services that have to record one entry per
        /// token, and have protections against "unreasonably large" batches of
        /// tokens.
        uint96 _max_batch_size;
    }
}

sol! {
    /// Emitted when the tokens from `from_token_id` to `to_token_id` are transferred from `from_address` to `to_address`.
    ///
    /// * `from_token_id` - First token being transferred.
    /// * `to_token_id` - Last token being transferred.
    /// * `from_address` - Address from which tokens will be transferred.
    /// * `to_address` - Address where the tokens will be transferred to.
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

/// An [`Erc721Consecutive`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc721`] contract [`erc721::Error`].
    Erc721(erc721::Error),
    /// Error type from checkpoint contract [`checkpoints::Error`].
    Checkpoints(checkpoints::Error),
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

unsafe impl TopLevelStorage for Erc721Consecutive {}

// ************** ERC-721 External **************

#[public]
impl IErc721 for Erc721Consecutive {
    type Error = Error;

    fn balance_of(&self, owner: Address) -> Result<U256, Error> {
        Ok(self.erc721.balance_of(owner)?)
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
        // TODO: Once the SDK supports the conversion,
        // use alloy_primitives::bytes!("") here.
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
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )?)
    }

    fn transfer_from(
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

    fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        self._approve(to, token_id, msg::sender(), true)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        Ok(self.erc721.set_approval_for_all(operator, approved)?)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, Error> {
        self._require_owned(token_id)?;
        Ok(self.erc721._get_approved(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721.is_approved_for_all(owner, operator)
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
        if self._sequential_burn.get(token_id) {
            Address::ZERO
        } else {
            // NOTE: Bounds already checked. No need for safe cast of token_id
            self._sequential_ownership.lower_lookup(U96::from(token_id)).into()
        }
    }

    /// Mint a batch of tokens with length `batch_size` for `to`.
    /// Returns the token id of the first token minted in the batch; if
    /// `batch_size` is 0, returns the number of consecutive ids minted so
    /// far.
    ///
    /// Requirements:
    ///
    /// * `batch_size` must not be greater than
    ///   [`Erc721Consecutive::_max_batch_size`].
    /// * The function is called in the constructor of the contract (directly or
    ///   indirectly).
    ///
    /// CAUTION: Does not emit a [Transfer] event.
    /// This is ERC-721 compliant as
    /// long as it is done inside of the constructor, which is enforced by
    /// this function.
    ///
    /// CAUTION: Does not invoke
    /// [`erc721::IERC721Receiver::on_erc_721_received`] on the receiver.
    ///
    /// # Arguments
    ///
    /// * `&self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If `batch_size` exceeds [`Erc721Consecutive::_max_batch_size`],
    /// then the error [`Error::ExceededMaxBatchMint`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`ConsecutiveTransfer`] event.
    #[cfg(all(test, feature = "std"))]
    fn _mint_consecutive(
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
            let last = next + batch_size - uint!(1_U96);
            self._sequential_ownership.push(last, to.into())?;

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
        };
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
    /// If token does not exist and `auth` is not `Address::ZERO`, then the
    /// error [`erc721::Error::NonexistentToken`] is returned.
    /// If `auth` is not `Address::ZERO` and `auth` does not have a right to
    /// approve this token, then the error
    /// [`erc721::Error::InsufficientApproval`] is returned.
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
        let previous_owner = self._update_base(to, token_id, auth)?;

        // if we burn
        if to.is_zero()
            // and the token_id was minted in a batch
            && token_id < U256::from(self._next_consecutive_id())
            // and the token was never marked as burnt
            && !self._sequential_burn.get(token_id)
        {
            // record burn
            self._sequential_burn.set(token_id);
        }

        Ok(previous_owner)
    }

    /// Returns the next token_id to mint using [`Self::_mint_consecutive`]. It
    /// will return [`Erc721Consecutive::_first_consecutive_id`] if no
    /// consecutive token_id has been minted before.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn _next_consecutive_id(&self) -> U96 {
        match self._sequential_ownership.latest_checkpoint() {
            None => self._first_consecutive_id(),
            Some((latest_id, _)) => latest_id + uint!(1_U96),
        }
    }

    /// Used to offset the first token id in
    /// [`Erc721Consecutive::_next_consecutive_id`].
    fn _first_consecutive_id(&self) -> U96 {
        self._first_consecutive_id.get()
    }

    /// Maximum size of consecutive token's batch.
    /// This is designed to limit stress on off-chain indexing services that
    /// have to record one entry per token, and have protections against
    /// "unreasonably large" batches of tokens.
    fn _max_batch_size(&self) -> U96 {
        self._max_batch_size.get()
    }
}

// ************** ERC-721 Internal **************

impl Erc721Consecutive {
    /// Transfers `token_id` from its current owner to `to`, or alternatively
    /// mints (or burns) if the current owner (or `to`) is the `Address::ZERO`.
    /// Returns the owner of the `token_id` before the update.
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
            self.erc721
                ._balances
                .setter(from)
                .sub_assign_unchecked(uint!(1_U256));
        }

        if !to.is_zero() {
            self.erc721
                ._balances
                .setter(to)
                .add_assign_unchecked(uint!(1_U256));
        }

        self.erc721._owners.setter(token_id).set(to);
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
    /// If `token_id` already exists, then the error
    /// [`erc721::Error::InvalidSender`] is returned.
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * `to` cannot be `Address::ZERO`.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
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

    /// Mints `token_id`, transfers it to `to`,
    /// and checks for `to`'s acceptance.
    ///
    /// An additional `data` parameter is forwarded to
    /// [`erc721::IERC721Receiver::on_erc_721_received`] to contract recipients.
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
    /// If `token_id` already exists, then the error
    /// [`erc721::Error::InvalidSender`] is returned.
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If [`erc721::IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`erc721::IERC721Receiver::on_erc_721_received`], which is called upon
    ///   a `safe_transfer`.
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
    /// If token does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
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
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If `token_id` does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    /// If the previous owner is not `from`, then  the error
    /// [`erc721::Error::IncorrectOwner`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `to` cannot be `Address::ZERO`.
    /// * The `token_id` token must be owned by `from`.
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
    /// invokes [`erc721::IERC721Receiver::on_erc_721_received`] on the
    /// receiver, and can be used to e.g. implement alternative mechanisms
    /// to perform token transfer, such as signature-based.
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
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If `token_id` does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    /// If the previous owner is not `from`, then the error
    /// [`erc721::Error::IncorrectOwner`] is returned.
    ///
    /// # Requirements:
    ///
    /// * The `token_id` token must exist and be owned by `from`.
    /// * `to` cannot be `Address::ZERO`.
    /// * `from` cannot be `Address::ZERO`.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`erc721::IERC721Receiver::on_erc_721_received`], which is called upon
    ///   a `safe_transfer`.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn _safe_transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
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
    /// `Address::ZERO`, then this function will check that `auth` is either
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
    /// If the token does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    /// If `auth` does not have a right to approve this token, then the error
    /// [`erc721::Error::InvalidApprover`] is returned.
    ///
    /// # Events
    ///
    /// Emits an [`Approval`] event.
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
                return Err(erc721::Error::InvalidApprover(
                    ERC721InvalidApprover { approver: auth },
                )
                .into());
            }

            if emit_event {
                evm::log(Approval { owner, approved: to, token_id });
            }
        }

        self.erc721._token_approvals.setter(token_id).set(to);
        Ok(())
    }

    /// Reverts if the `token_id` doesn't have a current owner (it hasn't been
    /// minted, or it has been burned). Returns the owner.
    ///
    /// Overrides to ownership logic should be done to
    /// [`Self::_owner_of`].
    ///
    /// # Errors
    ///
    /// If token does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
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

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use crate::token::{
        erc721,
        erc721::{
            extensions::consecutive::{
                ERC721ExceededMaxBatchMint, Erc721Consecutive, Error, U96,
            },
            tests::random_token_id,
            ERC721IncorrectOwner, ERC721InvalidApprover, ERC721InvalidReceiver,
            ERC721InvalidSender, ERC721NonexistentToken, IErc721,
        },
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");

    fn init(
        contract: &mut Erc721Consecutive,
        receivers: Vec<Address>,
        batches: Vec<U96>,
    ) -> Vec<U96> {
        contract._first_consecutive_id.set(uint!(0_U96));
        contract._max_batch_size.set(uint!(5000_U96));
        receivers
            .into_iter()
            .zip(batches)
            .map(|(to, batch_size)| {
                contract
                    ._mint_consecutive(to, batch_size)
                    .expect("should mint consecutively")
            })
            .collect()
    }

    #[motsu::test]
    fn mints(contract: Erc721Consecutive) {
        let alice = msg::sender();

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let init_tokens_count = uint!(10_U96);
        init(contract, vec![alice], vec![init_tokens_count]);

        let balance1 = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");
        assert_eq!(balance1, initial_balance + U256::from(init_tokens_count));

        // Check non-consecutive mint.
        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint a token for Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance2 = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(balance2, balance1 + uint!(1_U256));
    }

    #[motsu::test]
    fn error_when_minting_token_id_twice(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let token_id = random_token_id();
        contract
            ._mint(alice, token_id)
            .expect("should mint the token a first time");
        let err = contract
            ._mint(alice, token_id)
            .expect_err("should not mint a token with `token_id` twice");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::InvalidSender(ERC721InvalidSender {
                sender: Address::ZERO
            }))
        ));
    }

    #[motsu::test]
    fn error_when_minting_token_invalid_receiver(contract: Erc721Consecutive) {
        let invalid_receiver = Address::ZERO;

        let token_id = random_token_id();

        let err = contract
            ._mint(invalid_receiver, token_id)
            .expect_err("should not mint a token for invalid receiver");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            })) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn error_when_to_is_zero(contract: Erc721Consecutive) {
        let err = contract
            ._mint_consecutive(Address::ZERO, uint!(11_U96))
            .expect_err("should not mint consecutive");
        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO }
            ))
        ));
    }

    #[motsu::test]
    fn error_when_exceed_batch_size(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let batch_size = contract._max_batch_size() + uint!(1_U96);
        let err = contract
            ._mint_consecutive(alice, batch_size)
            .expect_err("should not mint consecutive");
        assert!(matches!(
            err,
            Error::ExceededMaxBatchMint(ERC721ExceededMaxBatchMint {
                batch_size,
                max_batch
            })
            if batch_size == U256::from(batch_size) && max_batch == U256::from(contract._max_batch_size())
        ));
    }

    #[motsu::test]
    fn transfers_from(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let bob = BOB;

        // Mint batches of 1000 tokens to Alice and Bob.
        let [first_consecutive_token_id, _] = init(
            contract,
            vec![alice, bob],
            vec![uint!(1000_U96), uint!(1000_U96)],
        )
        .try_into()
        .expect("should have two elements in return vec");

        // Transfer first consecutive token from Alice to Bob.
        contract
            .transfer_from(alice, bob, U256::from(first_consecutive_token_id))
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(U256::from(first_consecutive_token_id))
            .expect("token should be owned");
        assert_eq!(owner, bob);

        // Check that balances changed.
        let alice_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
        let bob_balance =
            contract.balance_of(bob).expect("should return the balance of Bob");
        assert_eq!(bob_balance, uint!(1000_U256) + uint!(1_U256));

        // Check non-consecutive mint.
        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint a token to Alice");
        let alice_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256));

        // Check transfer of the token that wasn't minted consecutive.
        contract
            .transfer_from(alice, BOB, token_id)
            .expect("should transfer a token from Alice to Bob");
        let alice_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));
    }

    #[motsu::test]
    fn burns(contract: Erc721Consecutive) {
        let alice = msg::sender();

        // Mint batch of 1000 tokens to Alice.
        let [first_consecutive_token_id] =
            init(contract, vec![alice], vec![uint!(1000_U96)])
                .try_into()
                .expect("should have two elements in return vec");

        // Check consecutive token burn.
        contract
            ._burn(U256::from(first_consecutive_token_id))
            .expect("should burn token");

        let alice_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256) - uint!(1_U256));

        let err = contract
            .owner_of(U256::from(first_consecutive_token_id))
            .expect_err("token should not exist");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken(ERC721NonexistentToken { token_id }))
            if token_id == U256::from(first_consecutive_token_id)
        ));

        // Check non-consecutive token burn.
        let non_consecutive_token_id = random_token_id();
        contract
            ._mint(alice, non_consecutive_token_id)
            .expect("should mint a token to Alice");
        let owner = contract
            .owner_of(non_consecutive_token_id)
            .expect("should return owner of the token");
        assert_eq!(owner, alice);
        let alice_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");
        assert_eq!(alice_balance, uint!(1000_U256));

        contract._burn(non_consecutive_token_id).expect("should burn token");

        let err = contract
            .owner_of(U256::from(non_consecutive_token_id))
            .expect_err("token should not exist");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken(ERC721NonexistentToken { token_id }))
            if token_id == U256::from(non_consecutive_token_id)
        ));

        // After being burnt the token should not be burnt again.
        let non_existent_token = non_consecutive_token_id;
        let err = contract
            ._burn(non_existent_token)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            })) if t_id == non_existent_token
        ));
    }

    #[motsu::test]
    fn safe_transfer_from(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint a token to Alice");

        contract
            .safe_transfer_from(alice, BOB, token_id)
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");

        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_token(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let token_id = random_token_id();
        contract._mint(BOB, token_id).expect("should mint token to Bob");
        contract.erc721._token_approvals.setter(token_id).set(alice);
        contract
            .safe_transfer_from(BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_incorrect_owner(
        contract: Erc721Consecutive,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        contract._mint(alice, token_id).expect("should mint a token to Alice");

        let err = contract
            .safe_transfer_from(DAVE, BOB, token_id)
            .expect_err("should not transfer from incorrect owner");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            })) if sender == DAVE && t_id == token_id && owner == alice
        ));
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_nonexistent_token(
        contract: Erc721Consecutive,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err = contract
            ._safe_transfer(alice, BOB, token_id, vec![0, 1, 2, 3].into())
            .expect_err("should not transfer a non-existent token");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            })) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_safe_transfer_to_invalid_receiver(
        contract: Erc721Consecutive,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, token_id).expect("should mint a token to Alice");

        let err = contract
            .safe_transfer_from(alice, invalid_receiver, token_id)
            .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            })) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn safe_transfers_from_with_data(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint a token to Alice");

        contract
            .safe_transfer_from_with_data(
                alice,
                BOB,
                token_id,
                vec![0, 1, 2, 3].into(),
            )
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");

        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_to_invalid_receiver(
        contract: Erc721Consecutive,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        contract._mint(alice, token_id).expect("should mint a token to Alice");

        let err = contract
            ._safe_transfer(
                alice,
                invalid_receiver,
                token_id,
                vec![0, 1, 2, 3].into(),
            )
            .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            })) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_internal_safe_transfer_from_incorrect_owner(
        contract: Erc721Consecutive,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        contract._mint(alice, token_id).expect("should mint a token to Alice");

        let err = contract
            ._safe_transfer(DAVE, BOB, token_id, vec![0, 1, 2, 3].into())
            .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            })) if sender == DAVE && t_id == token_id && owner == alice
        ));
    }

    #[motsu::test]
    fn safe_mints(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let token_id = random_token_id();

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        contract
            ._safe_mint(alice, token_id, vec![0, 1, 2, 3].into())
            .expect("should mint a token for Alice");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);

        let balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(initial_balance + uint!(1_U256), balance);
    }

    #[motsu::test]
    fn approves(contract: Erc721Consecutive) {
        let alice = msg::sender();
        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint a token");
        contract
            .approve(BOB, token_id)
            .expect("should approve Bob for operations on token");
        assert_eq!(contract.erc721._token_approvals.get(token_id), BOB);
    }

    #[motsu::test]
    fn error_when_approve_for_nonexistent_token(contract: Erc721Consecutive) {
        let token_id = random_token_id();
        let err = contract
            .approve(BOB, token_id)
            .expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            })) if token_id == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_by_invalid_approver(contract: Erc721Consecutive) {
        let token_id = random_token_id();
        contract._mint(BOB, token_id).expect("should mint a token");

        let err = contract
            .approve(DAVE, token_id)
            .expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::InvalidApprover(ERC721InvalidApprover {
                approver
            })) if approver == msg::sender()
        ));
    }

    #[motsu::test]
    fn approval_for_all(contract: Erc721Consecutive) {
        let alice = msg::sender();
        contract
            .erc721
            ._operator_approvals
            .setter(alice)
            .setter(BOB)
            .set(false);

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
    fn error_when_get_approved_of_nonexistent_token(
        contract: Erc721Consecutive,
    ) {
        let token_id = random_token_id();
        let err = contract
            .get_approved(token_id)
            .expect_err("should not return approved for a non-existent token");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            })) if token_id == t_id
        ));
    }
}
