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
//! IMPORTANT: Function [`Erc721Consecutive::_mint_consecutive`] is not suitable
//! to be called from constructor. Because of stylus sdk limitation. Function
//! [`Erc721Consecutive::_stop_mint_consecutive`] should be called to end
//! consecutive mint of tokens. After that, minting a token
//! with [`Erc721Consecutive::_mint`] will be possible.
//!
//! IMPORTANT: This extension does not call the [`Erc721::_update`] function for
//! tokens minted in batch. Any logic added to this function through overrides
//! will not be triggered when token are minted in batch. You may want to also
//! override [`Erc721Consecutive::_increaseBalance`] or
//! [`Erc721Consecutive::_mintConsecutive`] to account for these mints.
//!
//! IMPORTANT: When overriding [`Erc721Consecutive::_mintConsecutive`], be
//! careful about call ordering. [`Erc721Consecutive::owner_of`] may return
//! invalid values during the [`Erc721Consecutive::_mintConsecutive`]
//! execution if the super call is not called first. To be safe, execute the
//! super call before your custom logic.

use alloc::vec;

use alloy_primitives::{uint, Address, U128, U256};
use alloy_sol_types::sol;
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{
    abi::Bytes, call::MethodError, evm, msg, prelude::TopLevelStorage,
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
            checkpoints::{Trace160, U96},
        },
    },
};

sol_storage! {
    /// State of an [`Erc721Consecutive`] token.
    pub struct Erc721Consecutive {
        /// Erc721 contract storage.
        Erc721 erc721;
        /// Checkpoint library contract for sequential ownership.
        Trace160 _sequential_ownership;
        /// BitMap library contract for sequential burn of tokens.
        BitMap _sequential_burn;
        /// Initialization marker. If true this means that consecutive mint was already triggered.
        bool _initialized
    }
}

sol! {
    /// Emitted when the tokens from `from_token_id` to `to_token_id` are transferred from `from_address` to `to_address`.
    ///
    /// * `from_token_id` - First token being transferred.
    /// * `to_token_id` - Last token being transferred.
    /// * `from_address` - Address from which tokens will be transferred.
    /// * `to_address` - Address where the tokens will be transferred to.
    event ConsecutiveTransfer(
        uint256 indexed from_token_id,
        uint256 to_token_id,
        address indexed from_address,
        address indexed to_address
    );
}

sol! {
    /// Batch mint is restricted to the constructor.
    /// Any batch mint not emitting the [`IERC721::Transfer`] event outside of the constructor
    /// is non ERC-721 compliant.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ForbiddenBatchMint();

    /// Exceeds the max number of mints per batch.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ExceededMaxBatchMint(uint256 batchSize, uint256 maxBatch);

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
    /// Any batch mint not emitting the [`IERC721::Transfer`] event outside of
    /// the constructor is non ERC-721 compliant.
    ForbiddenBatchMint(ERC721ForbiddenBatchMint),
    /// Exceeds the max amount of mints per batch.
    ExceededMaxBatchMint(ERC721ExceededMaxBatchMint),
    /// Individual minting is not allowed.
    ForbiddenMint(ERC721ForbiddenMint),
    /// Batch burn is not supported.
    ForbiddenBatchBurn(ERC721ForbiddenBatchBurn),
}

impl MethodError for erc721::Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

impl MethodError for checkpoints::Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

// TODO: add an option to override these constants

/// Maximum size of a batch of consecutive tokens. This is designed to limit
/// stress on off-chain indexing services that have to record one entry per
/// token, and have protections against "unreasonably large" batches of tokens.
pub const MAX_BATCH_SIZE: U96 = uint!(5000_U96);

/// Used to offset the first token id in
/// [`Erc721Consecutive::_next_consecutive_id`].
pub const FIRST_CONSECUTIVE_ID: U96 = uint!(0_U96);

/// Consecutive extension related implementation:
impl Erc721Consecutive {
    /// Override of [`Erc721::_owner_of_inner`] that checks the sequential
    /// ownership structure for tokens that have been minted as part of a
    /// batch, and not yet transferred.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    pub fn _owner_of_inner(&self, token_id: U256) -> Address {
        let owner = self.erc721._owner_of_inner(token_id);
        // If token is owned by the core, or beyond consecutive range, return
        // base value.
        if owner != Address::ZERO
            || token_id < U256::from(FIRST_CONSECUTIVE_ID)
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

    /// Mint a batch of tokens of length `batch_size` for `to`. Returns the
    /// token id of the first token minted in the batch; if `batchSize` is
    /// 0, returns the number of consecutive ids minted so far.
    ///
    /// Requirements:
    ///
    /// - `batchSize` must not be greater than [`MAX_BATCH_SIZE`].
    /// - The function is called in the constructor of the contract (directly or
    ///   indirectly).
    ///
    /// CAUTION: Does not emit a `Transfer` event. This is ERC-721 compliant as
    /// long as it is done inside of the constructor, which is enforced by
    /// this function.
    ///
    /// CAUTION: Does not invoke `onERC721Received` on the receiver.
    ///
    /// # Arguments
    ///
    /// * `&self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If `to` is [`Address::ZERO`], then the error
    /// [`rc721::Error::InvalidReceiver`] is returned.
    /// If `batch_size` exceeds [`MAX_BATCH_SIZE`], then the error
    /// [`Error::ERC721ExceededMaxBatchMint`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`ConsecutiveTransfer`] event.
    pub fn _mint_consecutive(
        &mut self,
        to: Address,
        batch_size: U96,
    ) -> Result<U96, Error> {
        let next = self._next_consecutive_id();

        // Minting a batch of size 0 is a no-op.
        if batch_size > U96::ZERO {
            if self._initialized.get() {
                return Err(ERC721ForbiddenBatchMint {}.into());
            }

            if to.is_zero() {
                return Err(erc721::Error::InvalidReceiver(
                    ERC721InvalidReceiver { receiver: Address::ZERO },
                )
                .into());
            }

            if batch_size > MAX_BATCH_SIZE {
                return Err(ERC721ExceededMaxBatchMint {
                    batchSize: U256::from(batch_size),
                    maxBatch: U256::from(MAX_BATCH_SIZE),
                }
                .into());
            }

            // Push an ownership checkpoint & emit event.
            let last = next + batch_size - uint!(1_U96);
            self._sequential_ownership.push(last, to.into())?;

            // The invariant required by this function is preserved because the
            // new sequentialOwnership checkpoint is attributing
            // ownership of `batch_size` new tokens to account `to`.
            self.erc721._increase_balance(to, U128::from(batch_size));

            evm::log(ConsecutiveTransfer {
                from_token_id: next.to::<U256>(),
                to_token_id: last.to::<U256>(),
                from_address: Address::ZERO,
                to_address: to,
            });
        };
        Ok(next)
    }

    /// Should be called to restrict consecutive mint after.
    /// After this function being called, every call to
    /// [`Self::_mint_consecutive`] will fail.
    ///
    /// # Arguments
    ///
    /// * `&self` - Write access to the contract's state.
    pub fn _stop_mint_consecutive(&mut self) {
        self._initialized.set(true);
    }

    /// Override of [`Erc721::_update`] that restricts normal minting to after
    /// construction.
    ///
    /// WARNING: Using [`Erc721Consecutive`] prevents minting during
    /// construction in favor of [`Erc721Consecutive::_mint_consecutive`].
    /// After construction,[`Erc721Consecutive::_mint_consecutive`] is no
    /// longer available and minting through [`Erc721Consecutive::_update`]
    /// becomes possible.
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
    /// If consecutive mint wasn't finished yet (function
    /// [`Self::_stop_mint_consecutive`] wasn't called) error
    /// [`Error::ERC721ForbiddenMint`] is returned.
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
        let previous_owner = self.__update(to, token_id, auth)?;

        // only mint after construction.
        if previous_owner == Address::ZERO && !self._initialized.get() {
            return Err(ERC721ForbiddenMint {}.into());
        }

        // record burn.
        if to == Address::ZERO // if we burn.
            && token_id < U256::from(self._next_consecutive_id()) // and the tokenId was minted in a batch.
            && !self._sequential_burn.get(token_id)
        // and the token was never marked as burnt.
        {
            self._sequential_burn.set(token_id);
        }

        Ok(previous_owner)
    }

    /// Returns the next tokenId to mint using [`Self::_mint_consecutive`]. It
    /// will return [`FIRST_CONSECUTIVE_ID`] if no consecutive tokenId has
    /// been minted before.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn _next_consecutive_id(&self) -> U96 {
        match self._sequential_ownership.latest_checkpoint() {
            None => FIRST_CONSECUTIVE_ID,
            Some((latest_id, _)) => latest_id + uint!(1_U96),
        }
    }
}

unsafe impl TopLevelStorage for Erc721Consecutive {}

#[external]
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
            &data,
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
        // verifies that the token exists (`from != 0`). Therefore, it is
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
        Ok(self.erc721._get_approved_inner(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721.is_approved_for_all(owner, operator)
    }
}

// ERC-721 related implementation:
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
    /// also [`Self::_increase_balance`].
    fn __update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let from = self._owner_of_inner(token_id);

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
    /// [`Error::InvalidSender`] is returned.
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
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
    /// [`IERC721Receiver::on_erc_721_received`] to contract recipients.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Self::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// If `token_id` already exists, then the error
    /// [`Error::InvalidSender`] is returned.
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon a
    ///   `safe_transfer`.
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
            &data,
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
    /// [`Error::NonexistentToken`] is returned.
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
    /// [`Error::InvalidReceiver`] is returned.
    /// If `token_id` does not exist, then the error
    /// [`Error::ERC721NonexistentToken`] is returned.
    /// If the previous owner is not `from`, then  the error
    /// [`Error::IncorrectOwner`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `to` cannot be the zero address.
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
    ///   [`Self::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`Error::InvalidReceiver`] is returned.
    /// If `token_id` does not exist, then the error
    /// [`Error::ERC721NonexistentToken`] is returned.
    /// If the previous owner is not `from`, then the error
    /// [`Error::IncorrectOwner`] is returned.
    ///
    /// # Requirements:
    ///
    /// * The `token_id` token must exist and be owned by `from`.
    /// * `to` cannot be the zero address.
    /// * `from` cannot be the zero address.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`IERC721Receiver::on_erc_721_received`], which is called upon a
    ///   `safe_transfer`.
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
            &data,
        )?)
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
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    /// If `auth` does not have a right to approve this token, then the error
    /// [`Error::InvalidApprover`] is returned.
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
    /// [`Self::_owner_of_inner`].
    ///
    /// # Errors
    ///
    /// If token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    pub fn _require_owned(&self, token_id: U256) -> Result<Address, Error> {
        let owner = self._owner_of_inner(token_id);
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
    use stylus_sdk::{msg, prelude::StorageType};

    use crate::{
        token::{
            erc721,
            erc721::{
                extensions::consecutive::{
                    ERC721ExceededMaxBatchMint, ERC721ForbiddenBatchMint,
                    Erc721Consecutive, Error, MAX_BATCH_SIZE,
                },
                tests::random_token_id,
                ERC721InvalidReceiver, ERC721NonexistentToken, Erc721, IErc721,
            },
        },
        utils::structs::{
            bitmap::BitMap,
            checkpoints::{Trace160, U96},
        },
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    fn init(
        contract: &mut Erc721Consecutive,
        receivers: Vec<Address>,
        batches: Vec<U96>,
    ) -> Vec<U96> {
        let token_ids = receivers
            .into_iter()
            .zip(batches)
            .map(|(to, batch_size)| {
                contract
                    ._mint_consecutive(to, batch_size)
                    .expect("should mint consecutively")
            })
            .collect();
        contract._stop_mint_consecutive();
        token_ids
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
    fn error_when_not_minted_consecutive(contract: Erc721Consecutive) {
        let alice = msg::sender();

        init(contract, vec![alice], vec![uint!(10_U96)]);

        let err = contract
            ._mint_consecutive(BOB, uint!(11_U96))
            .expect_err("should not mint consecutive");
        assert!(matches!(
            err,
            Error::ForbiddenBatchMint(ERC721ForbiddenBatchMint {})
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
        let batch_size = MAX_BATCH_SIZE + uint!(1_U96);
        let err = contract
            ._mint_consecutive(alice, batch_size)
            .expect_err("should not mint consecutive");
        assert!(matches!(
            err,
            Error::ExceededMaxBatchMint(ERC721ExceededMaxBatchMint {
                batchSize,
                maxBatch
            })
            if batchSize == U256::from(batch_size) && maxBatch == U256::from(MAX_BATCH_SIZE)
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

        // Check non-consecutive mint
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
    }
}
