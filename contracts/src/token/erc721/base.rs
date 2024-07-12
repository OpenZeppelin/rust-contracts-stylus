use alloc::vec;
use core::marker::PhantomData;

use alloy_primitives::{fixed_bytes, uint, Address, FixedBytes, U128, U256};
use stylus_sdk::{
    abi::Bytes, alloy_sol_types::sol, call::Call, evm, msg, prelude::*,
};

use crate::{
    token::erc721::{
        traits::{IErc721, IErc721Virtual},
        Error,
    },
    utils::math::storage::{AddAssignUnchecked, SubAssignUnchecked},
};

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
    /// For example, `Address::ZERO` is a forbidden owner in [`crate::token::erc721::Erc721`].
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

sol_interface! {
    /// [`crate::token::erc721::Erc721`] token receiver interface.
    ///
    /// Interface for any contract that wants to support `safe_transfers`
    /// from [`crate::token::erc721::Erc721`] asset contracts.
    interface IERC721Receiver {
        /// Whenever an [`crate::token::erc721::Erc721`] `token_id` token is transferred
        /// to this contract via [`crate::token::erc721::Erc721::safe_transfer_from`].
        ///
        /// It must return its function selector to confirm the token transfer.
        /// If any other value is returned or the interface is not implemented
        /// by the recipient, the transfer will be reverted.
        #[allow(missing_docs)]
        function onERC721Received(
            address operator,
            address from,
            uint256 token_id,
            bytes calldata data
        ) external returns (bytes4);
    }
}

sol_storage! {
    /// State of an [`crate::token::erc721::Erc721`] token.
    #[cfg_attr(all(test, feature = "std"), derive(motsu::DefaultStorageLayout))]
    pub struct Erc721<V: IErc721Virtual> {
        /// Maps tokens to owners.
        mapping(uint256 => address) _owners;
        /// Maps users to balances.
        mapping(address => uint256) _balances;
        /// Maps tokens to approvals.
        mapping(uint256 => address) _token_approvals;
        /// Maps owners to a mapping of operator approvals.
        mapping(address => mapping(address => bool)) _operator_approvals;
        PhantomData<V> _phantom_data;
    }
}

#[external]
impl<V: IErc721Virtual> IErc721 for Erc721<V> {
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
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        // TODO: use bytes! macro later
        Self::safe_transfer_from_with_data(
            storage,
            from,
            to,
            token_id,
            alloc::vec![].into(),
        )
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        Self::transfer_from(storage, from, to, token_id)?;
        Self::_check_on_erc721_received(
            storage,
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )
    }

    fn transfer_from(
        storage: &mut impl TopLevelStorage,
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
        let previous_owner =
            Self::update(storage, to, token_id, msg::sender())?;
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
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        V::approve::<V>(storage, to, token_id, msg::sender(), true)
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
        Ok(self._get_approved_inner(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self._operator_approvals.get(owner).get(operator)
    }
}

pub struct Erc721Override;
impl IErc721Virtual for Erc721Override {
    type Base = Self;

    fn update<V: IErc721Virtual>(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        let base = storage.inner::<Erc721<V>>();
        let from = base._owner_of_inner(token_id);

        // Perform (optional) operator check.
        if !auth.is_zero() {
            base._check_authorized(from, auth, token_id)?;
        }

        // Execute the update.
        if !from.is_zero() {
            // Clear approval. No need to re-authorize or emit the Approval
            // event.
            V::approve::<V>(
                storage,
                Address::ZERO,
                token_id,
                Address::ZERO,
                false,
            )?;
            storage
                .inner_mut::<Erc721<V>>()
                ._balances
                .setter(from)
                .sub_assign_unchecked(U256::from(1));
        }

        if !to.is_zero() {
            storage
                .inner_mut::<Erc721<V>>()
                ._balances
                .setter(to)
                .add_assign_unchecked(U256::from(1));
        }

        storage.inner_mut::<Erc721<V>>()._owners.setter(token_id).set(to);

        evm::log(Transfer { from, to, token_id });

        Ok(from)
    }

    fn safe_transfer<V: IErc721Virtual>(
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        Erc721::<V>::_transfer(storage, from, to, token_id)?;
        Erc721::<V>::_check_on_erc721_received(
            storage,
            msg::sender(),
            from,
            to,
            token_id,
            data,
        )
    }

    fn approve<V: IErc721Virtual>(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
        emit_event: bool,
    ) -> Result<(), Error> {
        let storage: &mut Erc721<V> = storage.inner_mut();
        // Avoid reading the owner unless necessary.
        if emit_event || !auth.is_zero() {
            let owner = storage._require_owned(token_id)?;

            // We do not use [`Self::_is_authorized`] because single-token
            // approvals should not be able to call `approve`.
            if !auth.is_zero()
                && owner != auth
                && !storage.is_approved_for_all(owner, auth)
            {
                return Err(ERC721InvalidApprover { approver: auth }.into());
            }

            if emit_event {
                evm::log(Approval { owner, approved: to, token_id });
            }
        }

        storage._token_approvals.setter(token_id).set(to);
        Ok(())
    }
}

impl<V: IErc721Virtual> Erc721<V> {
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
    pub fn _owner_of_inner(&self, token_id: U256) -> Address {
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
                || self._get_approved_inner(token_id) == spender)
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
    /// If the token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
    /// If `spender` does not have the right to approve, then the error
    /// [`Error::InsufficientApproval`] is returned.
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
    /// be paired with an override of the [`Self::_owner_of_inner`] function to
    /// resolve the ownership of the corresponding tokens so that balances and
    /// ownership remain consistent with one another.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Account to increase balance.
    /// * `value` - The number of tokens to increase balance.
    // TODO: Right now this function is pointless since it is not used.
    // But once we will be able to override internal functions,
    // it will make a difference.
    pub fn _increase_balance(&mut self, account: Address, value: U128) {
        self._balances.setter(account).add_assign_unchecked(U256::from(value));
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
    pub fn _mint(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner =
            V::update::<V>(storage, to, token_id, Address::ZERO)?;
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
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        Self::_mint(storage, to, token_id)?;
        Self::_check_on_erc721_received(
            storage,
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
    pub fn _burn(
        storage: &mut impl TopLevelStorage,
        token_id: U256,
    ) -> Result<(), Error> {
        let previous_owner =
            V::update::<V>(storage, Address::ZERO, token_id, Address::ZERO)?;
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
        storage: &mut impl TopLevelStorage,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(
                ERC721InvalidReceiver { receiver: Address::ZERO }.into()
            );
        }

        let previous_owner =
            V::update::<V>(storage, to, token_id, Address::ZERO)?;
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
    /// If `operator` is `Address::ZERO`, then the error
    /// [`Error::InvalidOperator`] is returned.
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
            return Err(ERC721NonexistentToken { token_id }.into());
        }
        Ok(owner)
    }

    /// Performs an acceptance check for the provided `operator` by calling
    /// [`IERC721Receiver::on_erc_721_received`] on the `to` address. The
    /// `operator` is generally the address that initiated the token transfer
    /// (i.e. `msg::sender()`).
    ///
    /// The acceptance call is not executed and treated as a no-op
    /// if the target address doesn't contain code (i.e. an EOA).
    /// Otherwise, the recipient must implement
    /// [`IERC721Receiver::on_erc_721_received`] and return the
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
    /// If [`IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`Error::InvalidReceiver`] is returned.
    pub fn _check_on_erc721_received(
        storage: &mut impl TopLevelStorage,
        operator: Address,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        const IERC721RECEIVER_INTERFACE_ID: FixedBytes<4> =
            fixed_bytes!("150b7a02");

        // FIXME: Cleanup this code once it's covered in the test suite.
        if to.has_code() {
            let call = Call::new_in(storage);
            return match IERC721Receiver::new(to).on_erc_721_received(
                call,
                operator,
                from,
                token_id,
                data.to_vec(),
            ) {
                Ok(result) => {
                    if result == IERC721RECEIVER_INTERFACE_ID {
                        Ok(())
                    } else {
                        Err(ERC721InvalidReceiver { receiver: to }.into())
                    }
                }
                Err(_) => Err(ERC721InvalidReceiver { receiver: to }.into()),
            };
        }
        Ok(())
    }

    // TODO#q: develop this feature to autoimpl
    fn update(
        storage: &mut impl TopLevelStorage,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        V::update::<V>(storage, to, token_id, auth)
    }
}

#[cfg(all(test, feature = "std"))]
pub mod tests {
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::{msg, storage::TopLevelStorage};

    use crate::token::erc721::{
        base::Erc721Override as Override,
        traits::{IErc721, IErc721Virtual},
        ERC721IncorrectOwner, ERC721InsufficientApproval,
        ERC721InvalidApprover, ERC721InvalidOperator, ERC721InvalidOwner,
        ERC721InvalidReceiver, ERC721InvalidSender, ERC721NonexistentToken,
        Erc721, Error,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");
    const DAVE: Address = address!("0BB78F7e7132d1651B4Fd884B7624394e92156F1");

    pub fn random_token_id() -> U256 {
        let num: u32 = rand::random();
        U256::from(num)
    }

    unsafe impl TopLevelStorage for Erc721<Override> {}

    #[motsu::test]
    fn error_when_checking_balance_of_invalid_owner(
        contract: Erc721<Override>,
    ) {
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
    fn balance_of_zero_balance(contract: Erc721<Override>) {
        let owner = msg::sender();
        let balance =
            contract.balance_of(owner).expect("should return `U256::ZERO`");
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn error_when_checking_owner_of_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let token_id = random_token_id();

        let err = contract
            .owner_of(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn mints(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        Erc721::<Override>::_mint(contract, alice, token_id)
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
    fn error_when_minting_token_id_twice(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint the token a first time");
        let err = Erc721::<Override>::_mint(contract, alice, token_id)
            .expect_err("should not mint a token with `token_id` twice");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn safe_mints(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        Erc721::<Override>::_safe_mint(
            contract,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
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
    fn error_when_safe_mint_token_id_twice(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint the token a first time");

        let err = Erc721::<Override>::_safe_mint(
            contract,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not mint a token with `token_id` twice");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC721InvalidSender { sender: Address::ZERO })
        ));
    }

    #[motsu::test]
    fn error_when_safe_mint_invalid_receiver(contract: Erc721<Override>) {
        let invalid_receiver = Address::ZERO;

        let token_id = random_token_id();

        let err = Erc721::<Override>::_safe_mint(
            contract,
            invalid_receiver,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not mint a token for invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    fn transfers_from(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");
        Erc721::<Override>::transfer_from(contract, alice, BOB, token_id)
            .expect("should transfer a token from Alice to Bob");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn transfers_from_approved_token(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        contract._token_approvals.setter(token_id).set(alice);
        Erc721::<Override>::transfer_from(contract, BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_from_approved_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");

        // As we cannot change `msg::sender`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert_eq!(approved_for_all, true);

        Erc721::<Override>::transfer_from(contract, BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_to_invalid_receiver(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::transfer_from(
            contract,
            alice,
            invalid_receiver,
            token_id,
        )
        .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_from_incorrect_owner(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err =
            Erc721::<Override>::transfer_from(contract, DAVE, BOB, token_id)
                .expect_err(
                    "should not transfer the token from incorrect owner",
                );
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == token_id && owner == alice
        ));

        // FIXME: this check should pass
        // TODO: confirm in E2E tests that owner is not changed: #93
        // let owner = contract
        // .owner_of(token_id)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_with_insufficient_approval(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        let err =
            Erc721::<Override>::transfer_from(contract, BOB, alice, token_id)
                .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                    operator,
                    token_id: t_id,
            }) if operator == alice && t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_transfer_from_transfers_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err =
            Erc721::<Override>::transfer_from(contract, alice, BOB, token_id)
                .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                    token_id: t_id,
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn safe_transfers_from(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        Erc721::<Override>::safe_transfer_from(contract, alice, BOB, token_id)
            .expect("should transfer a token from Alice to Bob");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");

        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_token(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        contract._token_approvals.setter(token_id).set(alice);
        Erc721::<Override>::safe_transfer_from(contract, BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_from_approved_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert_eq!(approved_for_all, true);

        Erc721::<Override>::safe_transfer_from(contract, BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_to_invalid_receiver(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::safe_transfer_from(
            contract,
            alice,
            invalid_receiver,
            token_id,
        )
        .expect_err("should not transfer the token to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_from_incorrect_owner(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::safe_transfer_from(
            contract, DAVE, BOB, token_id,
        )
        .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                owner,
                sender,
                token_id: t_id
            }) if sender == DAVE && t_id == token_id && owner == alice
        ));

        // FIXME: this check should pass
        // TODO: confirm in E2E tests that owner is not changed: #93
        // let owner = contract
        // .owner_of(token_id)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_with_insufficient_approval(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        let err = Erc721::<Override>::safe_transfer_from(
            contract, BOB, alice, token_id,
        )
        .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id
            }) if operator == alice && t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_transfers_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err = Erc721::<Override>::safe_transfer_from(
            contract, alice, BOB, token_id,
        )
        .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn safe_transfers_from_with_data(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        Erc721::<Override>::safe_transfer_from_with_data(
            contract,
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
    fn safe_transfers_from_with_data_approved_token(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        contract._token_approvals.setter(token_id).set(alice);
        Erc721::<Override>::safe_transfer_from_with_data(
            contract,
            BOB,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_from_with_data_approved_for_all(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert_eq!(approved_for_all, true);

        Erc721::<Override>::safe_transfer_from_with_data(
            contract,
            BOB,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_to_invalid_receiver(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::safe_transfer_from_with_data(
            contract,
            alice,
            invalid_receiver,
            token_id,
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
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_from_incorrect_owner(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::safe_transfer_from_with_data(
            contract,
            DAVE,
            BOB,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == token_id && owner == alice

        ));

        // FIXME: this check should pass
        // TODO: confirm in E2E tests that owner is not changed: #93
        // let owner = contract
        // .owner_of(token_id)
        // .expect("should return the owner of the token");
        //
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_with_insufficient_approval(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        let err = Erc721::<Override>::safe_transfer_from_with_data(
            contract,
            BOB,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not transfer unapproved token");
        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id,
            }) if operator == alice && t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_safe_transfer_from_with_data_transfers_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err = Erc721::<Override>::safe_transfer_from_with_data(
            contract,
            alice,
            BOB,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn approves(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        Erc721::<Override>::approve(contract, BOB, token_id)
            .expect("should approve Bob for operations on token");
        assert_eq!(contract._token_approvals.get(token_id), BOB);
    }

    #[motsu::test]
    fn error_when_approve_for_nonexistent_token(contract: Erc721<Override>) {
        let token_id = random_token_id();
        let err = Erc721::<Override>::approve(contract, BOB, token_id)
            .expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if token_id == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_by_invalid_approver(contract: Erc721<Override>) {
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token");

        let err = Erc721::<Override>::approve(contract, DAVE, token_id)
            .expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::InvalidApprover(ERC721InvalidApprover {
                approver
            }) if approver == msg::sender()
        ));
    }

    #[motsu::test]
    fn approval_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        contract._operator_approvals.setter(alice).setter(BOB).set(false);

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
    fn error_when_approval_for_all_for_invalid_operator(
        contract: Erc721<Override>,
    ) {
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
    fn error_when_get_approved_of_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let token_id = random_token_id();
        let err = contract
            .get_approved(token_id)
            .expect_err("should not return approved for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if token_id == t_id
        ));
    }

    #[motsu::test]
    fn owner_of_inner_works(contract: Erc721<Override>) {
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token");

        let owner = contract._owner_of_inner(token_id);
        assert_eq!(BOB, owner);
    }

    #[motsu::test]
    fn owner_of_inner_nonexistent_token(contract: Erc721<Override>) {
        let token_id = random_token_id();
        let owner = contract._owner_of_inner(token_id);
        assert_eq!(Address::ZERO, owner);
    }

    #[motsu::test]
    fn get_approved_inner_nonexistent_token(contract: Erc721<Override>) {
        let token_id = random_token_id();
        let approved = contract._get_approved_inner(token_id);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn get_approved_inner_token_without_approval(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        let approved = contract._get_approved_inner(token_id);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn get_approved_inner_token_with_approval(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        Erc721::<Override>::approve(contract, BOB, token_id)
            .expect("should approve Bob for operations on token");

        let approved = contract._get_approved_inner(token_id);
        assert_eq!(BOB, approved);
    }

    #[motsu::test]
    fn get_approved_inner_token_with_approval_for_all(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");

        let approved = contract._get_approved_inner(token_id);
        assert_eq!(Address::ZERO, approved);
    }

    #[motsu::test]
    fn is_authorized_nonexistent_token(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let authorized = contract._is_authorized(alice, BOB, token_id);
        assert_eq!(false, authorized);
    }

    #[motsu::test]
    fn is_authorized_token_owner(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");

        let authorized = contract._is_authorized(alice, alice, token_id);
        assert_eq!(true, authorized);
    }

    #[motsu::test]
    fn is_authorized_without_approval(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");

        let authorized = contract._is_authorized(alice, BOB, token_id);
        assert_eq!(false, authorized);
    }

    #[motsu::test]
    fn is_authorized_with_approval(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        Erc721::<Override>::approve(contract, BOB, token_id)
            .expect("should approve Bob for operations on token");

        let authorized = contract._is_authorized(alice, BOB, token_id);
        assert_eq!(true, authorized);
    }

    #[motsu::test]
    fn is_authorized_with_approval_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");

        let authorized = contract._is_authorized(alice, BOB, token_id);
        assert_eq!(true, authorized);
    }

    #[motsu::test]
    fn check_authorized_nonexistent_token(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err = contract
            ._check_authorized(Address::ZERO, alice, token_id)
            .expect_err("should not pass for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn check_authorized_token_owner(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");

        let result = contract._check_authorized(alice, alice, token_id);

        assert!(result.is_ok());
    }

    #[motsu::test]
    fn check_authorized_without_approval(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");

        let err = contract
            ._check_authorized(alice, BOB, token_id)
            .expect_err("should not pass without approval");

        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                operator,
                token_id: t_id
            }) if operator == BOB && t_id == token_id
        ));
    }

    #[motsu::test]
    fn check_authorized_with_approval(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        Erc721::<Override>::approve(contract, BOB, token_id)
            .expect("should approve Bob for operations on token");

        let result = contract._check_authorized(alice, BOB, token_id);
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn check_authorized_with_approval_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        contract
            .set_approval_for_all(BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");

        let result = contract._check_authorized(alice, BOB, token_id);
        assert!(result.is_ok());
    }

    #[motsu::test]
    fn burns(contract: Erc721<Override>) {
        let alice = msg::sender();
        let one = uint!(1_U256);
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for Alice");

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let result = Erc721::<Override>::_burn(contract, token_id);
        let balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let err = contract
            .owner_of(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
                err,
                Error::NonexistentToken (ERC721NonexistentToken{
                    token_id: t_id
                }) if t_id == token_id
        ));

        assert!(result.is_ok());

        assert_eq!(initial_balance - one, balance);
    }

    #[motsu::test]
    fn error_when_get_approved_of_previous_approval_burned(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for Alice");
        Erc721::<Override>::approve(contract, BOB, token_id)
            .expect("should approve a token for Bob");

        Erc721::<Override>::_burn(contract, token_id)
            .expect("should burn previously minted token");

        let err = contract
            .get_approved(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_burn_nonexistent_token(contract: Erc721<Override>) {
        let token_id = random_token_id();

        let err = Erc721::<Override>::_burn(contract, token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn transfers(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");
        Erc721::<Override>::_transfer(contract, alice, BOB, token_id)
            .expect("should transfer a token from Alice to Bob");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, BOB);
    }

    #[motsu::test]
    fn transfers_approved_token(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        contract._token_approvals.setter(token_id).set(alice);
        Erc721::<Override>::_transfer(contract, BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn transfers_approved_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");

        // As we cannot change `msg::sender`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert_eq!(approved_for_all, true);

        Erc721::<Override>::_transfer(contract, BOB, alice, token_id)
            .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_to_invalid_receiver(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::_transfer(
            contract,
            alice,
            invalid_receiver,
            token_id,
        )
        .expect_err("should not transfer to invalid receiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC721InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_from_incorrect_owner(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Erc721::<Override>::_transfer(contract, DAVE, BOB, token_id)
            .expect_err("should not transfer from incorrect owner");

        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == token_id && owner == alice
        ));

        // FIXME: this check should pass
        // TODO: confirm in E2E tests that owner is not changed: #93
        // let owner = contract
        // .owner_of(token_id)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_transfer_transfers_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err = Erc721::<Override>::_transfer(contract, alice, BOB, token_id)
            .expect_err("should not transfer a non-existent token");
        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn safe_transfers_internal(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        Override::safe_transfer::<Override>(
            contract,
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
    fn safe_transfers_internal_approved_token(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");
        contract._token_approvals.setter(token_id).set(alice);
        Override::safe_transfer::<Override>(
            contract,
            BOB,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect("should transfer Bob's token to Alice");
        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn safe_transfers_internal_approved_for_all(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint token to Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let approved_for_all = contract.is_approved_for_all(BOB, alice);
        assert_eq!(approved_for_all, true);

        Override::safe_transfer::<Override>(
            contract,
            BOB,
            alice,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect("should transfer Bob's token to Alice");

        let owner = contract
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_safe_transfer_internal_ransfers_to_invalid_receiver(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let invalid_receiver = Address::ZERO;

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Override::safe_transfer::<Override>(
            contract,
            alice,
            invalid_receiver,
            token_id,
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
            .owner_of(token_id)
            .expect("should return the owner of the token");
        assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_internal_transfers_from_incorrect_owner(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token to Alice");

        let err = Override::safe_transfer::<Override>(
            contract,
            DAVE,
            BOB,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not transfer the token from incorrect owner");
        assert!(matches!(
            err,
            Error::IncorrectOwner(ERC721IncorrectOwner {
                sender,
                token_id: t_id,
                owner
            }) if sender == DAVE && t_id == token_id && owner == alice
        ));

        // FIXME: this check should pass
        // TODO: confirm in E2E tests that owner is not changed: #93
        // let owner = contract
        // .owner_of(token_id)
        // .expect("should return the owner of the token");
        // assert_eq!(alice, owner);
    }

    #[motsu::test]
    fn error_when_safe_transfer_internal_transfers_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        let err = Override::safe_transfer::<Override>(
            contract,
            alice,
            BOB,
            token_id,
            vec![0, 1, 2, 3].into(),
        )
        .expect_err("should not transfer a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id,
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn approves_internal(contract: Erc721<Override>) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token");
        Override::approve::<Override>(contract, BOB, token_id, alice, false)
            .expect("should approve Bob for operations on token");
        assert_eq!(contract._token_approvals.get(token_id), BOB);
    }

    #[motsu::test]
    fn error_when_approve_internal_for_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let token_id = random_token_id();
        let err = Override::approve::<Override>(
            contract,
            BOB,
            token_id,
            msg::sender(),
            false,
        )
        .expect_err("should not approve for a non-existent token");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if token_id == t_id
        ));
    }

    #[motsu::test]
    fn error_when_approve_internal_by_invalid_approver(
        contract: Erc721<Override>,
    ) {
        let alice = msg::sender();
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token");

        let err = Override::approve::<Override>(
            contract, DAVE, token_id, alice, false,
        )
        .expect_err("should not approve when invalid approver");

        assert!(matches!(
            err,
            Error::InvalidApprover(ERC721InvalidApprover {
                approver
            }) if approver == alice
        ));
    }

    #[motsu::test]
    fn approval_for_all_internal(contract: Erc721<Override>) {
        let alice = msg::sender();
        contract._operator_approvals.setter(alice).setter(BOB).set(false);

        contract
            ._set_approval_for_all(alice, BOB, true)
            .expect("should approve Bob for operations on all Alice's tokens");
        assert_eq!(contract.is_approved_for_all(alice, BOB), true);

        contract._set_approval_for_all(alice, BOB, false).expect(
            "should disapprove Bob for operations on all Alice's tokens",
        );
        assert_eq!(contract.is_approved_for_all(alice, BOB), false);
    }

    #[motsu::test]
    fn error_when_approval_for_all_internal_for_invalid_operator(
        contract: Erc721<Override>,
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
    fn require_owned_works(contract: Erc721<Override>) {
        let token_id = random_token_id();
        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token");

        let owner = contract
            ._require_owned(token_id)
            .expect("should return the owner of the token");

        assert_eq!(BOB, owner);
    }

    #[motsu::test]
    fn error_when_require_owned_for_nonexistent_token(
        contract: Erc721<Override>,
    ) {
        let token_id = random_token_id();
        let err = contract
            ._require_owned(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken(ERC721NonexistentToken {
                token_id: t_id
            }) if token_id == t_id
        ));
    }

    // TODO: think about [`Erc721::_update`] tests.

    // TODO: think about [`Erc721::_increase_balance`] tests
    // when it will be used.applicable.

    // TODO: add mock test for [`Erc721::_on_erc721_received`].
    // Should be done in integration tests.
}
