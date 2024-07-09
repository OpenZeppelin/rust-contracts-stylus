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
            Erc721, Error as Erc721Error, IERC721Receiver, IErc721, Transfer,
        },
    },
    utils::{
        math::storage::{AddAssignUnchecked, SubAssignUnchecked},
        structs::{
            bitmap::BitMap,
            checkpoints,
            checkpoints::{Trace160, U160, U96},
        },
    },
};

sol_storage! {
    /// State of an [`Erc72Erc721Consecutive`] token.
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Erc721Consecutive {
        Erc721 erc721;
        Trace160 _sequential_ownership;
        BitMap _sequentian_burn;
    }
}

sol! {
    /// Emitted when the tokens from `fromTokenId` to `toTokenId` are transferred from `fromAddress` to `toAddress`.
    event ConsecutiveTransfer(
        uint256 indexed fromTokenId,
        uint256 toTokenId,
        address indexed fromAddress,
        address indexed toAddress
    );
}

sol! {
    /// Batch mint is restricted to the constructor.
    /// Any batch mint not emitting the [`IERC721::Transfer`] event outside of the constructor
    /// is non ERC-721 compliant.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error ERC721ForbiddenBatchMint();

    /// Exceeds the max amount of mints per batch.
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

#[derive(SolidityError, Debug)]
pub enum Error {
    Erc721(erc721::Error),
    Checkpoints(checkpoints::Error),
    /// Batch mint is restricted to the constructor.
    /// Any batch mint not emitting the [`IERC721::Transfer`] event outside of
    /// the constructor is non ERC-721 compliant.
    Erc721ForbiddenBatchMint(ERC721ForbiddenBatchMint),
    /// Exceeds the max amount of mints per batch.
    Erc721ExceededMaxBatchMint(ERC721ExceededMaxBatchMint),
    /// Individual minting is not allowed.
    Erc721ForbiddenMint(ERC721ForbiddenMint),
    /// Batch burn is not supported.
    Erc721ForbiddenBatchBurn(ERC721ForbiddenBatchBurn),
}

impl MethodError for erc721::Error {
    fn encode(self) -> Vec<u8> {
        self.into()
    }
}

impl MethodError for checkpoints::Error {
    fn encode(self) -> Vec<u8> {
        self.into()
    }
}

// Maximum size of a batch of consecutive tokens. This is designed to limit
// stress on off-chain indexing services that have to record one entry per
// token, and have protections against "unreasonably large" batches of tokens.
const MAX_BATCH_SIZE: U96 = uint!(5000_U96);

// Used to offset the first token id in {_nextConsecutiveId}
const FIRST_CONSECUTIVE_ID: U96 = uint!(0_U96);

/// Consecutive extension related implementation:
impl Erc721Consecutive {
    /// Override that checks the sequential ownership structure for tokens that
    /// have been minted as part of a batch, and not yet transferred.
    pub fn _owner_of_inner(&self, token_id: U256) -> Address {
        let owner = self.__owner_of_inner(token_id);
        // If token is owned by the core, or beyond consecutive range, return
        // base value
        if owner != Address::ZERO
            || token_id > U256::from(U96::MAX)
            || token_id < U256::from(FIRST_CONSECUTIVE_ID)
        {
            return owner;
        }

        // Otherwise, check the token was not burned, and fetch ownership from
        // the anchors.
        // NOTE: no need for safe cast,
        if self._sequentian_burn.get(token_id) {
            Address::ZERO
        } else {
            self._sequential_ownership.lower_lookup(U96::from(token_id)).into()
        }
    }

    // Mint a batch of tokens of length `batchSize` for `to`. Returns the token
    // id of the first token minted in the batch; if `batchSize` is 0,
    // returns the number of consecutive ids minted so far.
    //
    // Requirements:
    //
    // - `batchSize` must not be greater than [`MAX_BATCH_SIZE`].
    // - The function is called in the constructor of the contract (directly or
    //   indirectly).
    //
    // CAUTION: Does not emit a `Transfer` event. This is ERC-721 compliant as
    // long as it is done inside of the constructor, which is enforced by
    // this function.
    //
    // CAUTION: Does not invoke `onERC721Received` on the receiver.
    //
    // Emits a [`ConsecutiveTransfer`] event.
    pub fn mint_consecutive(
        &mut self,
        to: Address,
        batch_size: u128,
    ) -> Result<u128, Error> {
        let batch_size = U96::from(batch_size);
        let next = self.next_consecutive_id();

        if batch_size > U96::ZERO {
            //TODO#q: check address of this and revert with ERC721ForbiddenBatchMint
            
            if to.is_zero() {
                return Err(Erc721Error::InvalidReceiver(
                    ERC721InvalidReceiver { receiver: Address::ZERO },
                )
                .into());
            }

            if batch_size > MAX_BATCH_SIZE.to() {
                return Err(ERC721ExceededMaxBatchMint {
                    batchSize: U256::from(batch_size),
                    maxBatch: U256::from(MAX_BATCH_SIZE),
                }
                .into());
            }

            let last = next + batch_size - uint!(1_U96);
            self._sequential_ownership
                .push(last, U160::from_be_bytes(to.into_array()))?;

            self.erc721._increase_balance(to, U128::from(batch_size));
            evm::log(ConsecutiveTransfer {
                fromTokenId: next.to::<U256>(),
                toTokenId: last.to::<U256>(),
                fromAddress: Address::ZERO,
                toAddress: to,
            });
        };
        Ok(next.to())
    }

    /// Override version that restricts normal minting to after construction.
    ///
    /// WARNING: Using [`Erc721Consecutive`] prevents minting during
    /// construction in favor of [`Erc721Consecutive::mint_consecutive`].
    /// After construction,[`Erc721Consecutive::mint_consecutive`] is no
    /// longer available and minting through [`Erc721Consecutive::_update`]
    /// becomes possible.
    pub fn _update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let previous_owner = self.__update(to, token_id, auth)?;

        // only mint after construction
        if previous_owner == Address::ZERO
        /* TODO#q: and address code is zero */
        {
            return Err(ERC721ForbiddenMint {}.into()); //
        }

        // record burn
        if to == Address::ZERO // if we burn
            && token_id < U256::from(self.next_consecutive_id()) // and the tokenId was minted in a batch
            && !self._sequentian_burn.get(token_id)
        // and the token was never marked as burnt
        {
            self._sequentian_burn.set(token_id);
        }

        Ok(previous_owner)
    }

    /// Returns the next tokenId to mint using {_mintConsecutive}. It will
    /// return [`FIRST_CONSECUTIVE_ID`] if no consecutive tokenId has been
    /// minted before.
    fn next_consecutive_id(&self) -> U96 {
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
        self.erc721.balance_of(owner).map_err(|e| e.into())
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
            return Err(Erc721Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into());
        }

        // Setting an "auth" argument enables the `_is_authorized` check which
        // verifies that the token exists (`from != 0`). Therefore, it is
        // not needed to verify that the return value is not 0 here.
        let previous_owner = self._update(to, token_id, msg::sender())?;
        if previous_owner != from {
            return Err(Erc721Error::IncorrectOwner(ERC721IncorrectOwner {
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

// erc721 related implementation:
impl Erc721Consecutive {
    /// Returns the owner of the `token_id`. Does NOT revert if the token
    /// doesn't exist.
    ///
    /// IMPORTANT: Any overrides to this function that add ownership of tokens
    /// not tracked by the core [`Erc721`] logic MUST be matched with the use
    /// of [`Self::_increase_balance`] to keep balances consistent with
    /// ownership. The invariant to preserve is that for any address `a` the
    /// value returned by `balance_of(a)` must be equal to the number of
    /// tokens such that `owner_of_inner(token_id)` is `a`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `token_id` - Token id as a number.
    #[must_use]
    pub fn __owner_of_inner(&self, token_id: U256) -> Address {
        self.erc721._owners.get(token_id)
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
    /// If token does not exist and `auth` is not `Address::ZERO`, then the
    /// error [`Error::NonexistentToken`] is returned.
    /// If `auth` is not `Address::ZERO` and `auth` does not have a right to
    /// approve this token, then the error
    /// [`Error::InsufficientApproval`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    pub fn __update(
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
            return Err(Erc721Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into());
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if !previous_owner.is_zero() {
            return Err(Erc721Error::InvalidSender(ERC721InvalidSender {
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
            return Err(Erc721Error::NonexistentToken(
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
            return Err(Erc721Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into());
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(Erc721Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
        } else if previous_owner != from {
            return Err(Erc721Error::IncorrectOwner(ERC721IncorrectOwner {
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
                return Err(Erc721Error::InvalidApprover(
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
            return Err(Erc721Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
        }
        Ok(owner)
    }
}
