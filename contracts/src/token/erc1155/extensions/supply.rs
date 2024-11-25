//! Extension of ERC-1155 that adds tracking of total supply per token id.
//!
//! Useful for scenarios where Fungible and Non-fungible tokens have to be
//! clearly identified. Note: While a total_supply of 1 might mean the
//! corresponding is an NFT, there are no guarantees that no other tokens
//! with the same id are not going to be minted.
//!
//! NOTE: This contract implies a global limit of 2**256 - 1 to the number
//! of tokens that can be minted.
//!
//! CAUTION: This extension should not be added in an upgrade to an already
//! deployed contract.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U256};
use stylus_sdk::{
    abi::Bytes,
    msg,
    prelude::{public, sol_storage},
    storage::TopLevelStorage,
};

use crate::{
    token::erc1155::{self, Erc1155, IErc1155},
    utils::math::storage::SubAssignUnchecked,
};

sol_storage! {
    /// State of an [`Erc1155Supply`] token.
    pub struct Erc1155Supply {
        /// ERC-1155 contract storage.
        Erc1155 erc1155;
        /// Mapping from token id to total supply.
        mapping(uint256 => uint256) _total_supply;
        /// Total supply of all token ids.
        uint256 _total_supply_all;
    }
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc1155Supply {}

#[public]
impl Erc1155Supply {
    /// Total value of tokens in with a given id.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    pub fn total_supply(&self, id: U256) -> U256 {
        self._total_supply.get(id)
    }

    /// Total value of tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "totalSupply")]
    pub fn total_supply_all(&self) -> U256 {
        *self._total_supply_all
    }

    /// Indicates whether any token exist with a given id, or not.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    pub fn exists(&self, id: U256) -> bool {
        self.total_supply(id) > U256::ZERO
    }

    /// Returns the value of tokens of type `id` owned by `account`.
    ///
    /// Re-export of [`Erc1155::balance_of`]
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account of the token's owner.
    /// * `id` - Token id as a number.
    pub fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.erc1155.balance_of(account, id)
    }

    /// Batched version of [`Erc1155::balance_of`].
    ///
    /// Re-export of [`Erc1155::balance_of_batch`]
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `accounts` - All account of the tokens' owner.
    /// * `ids` - All token identifiers.
    ///
    /// # Requirements
    ///
    /// * `accounts` and `ids` must have the same length.
    ///
    /// # Errors
    ///
    /// * If the length of `accounts` is not equal to the length of `ids`,
    /// then the error [`erc1155::Error::InvalidArrayLength`] is returned.
    pub fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, erc1155::Error> {
        self.erc1155.balance_of_batch(accounts, ids)
    }

    /// Grants or revokes permission to `operator`
    /// to transfer the caller's tokens, according to `approved`.
    ///
    /// Re-export of [`Erc1155::set_approval_for_all`]
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `operator` - Account to add to the set of authorized operators.
    /// * `approved` - Flag that determines whether or not permission will be
    ///   granted to `operator`. If true, this means `operator` will be allowed
    ///   to manage `msg::sender()`'s assets.
    ///
    /// # Errors
    ///
    /// * If `operator` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidOperator`] is returned.
    ///
    /// # Requirements
    ///
    /// * The `operator` cannot be the `Address::ZERO`.
    ///
    /// # Events
    ///
    /// Emits an [`erc1155::ApprovalForAll`] event.
    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), erc1155::Error> {
        self.erc1155.set_approval_for_all(operator, approved)
    }

    /// Returns true if `operator` is approved to transfer `account`'s
    /// tokens.
    ///
    /// Re-export of [`Erc1155::is_approved_for_all`]
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account of the token's owner.
    /// * `operator` - Account to be checked.
    pub fn is_approved_for_all(
        &self,
        account: Address,
        operator: Address,
    ) -> bool {
        self.erc1155.is_approved_for_all(account, operator)
    }

    /// Transfers a `value` amount of tokens of type `id` from `from` to
    /// `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id as a number.
    /// * `value` - Amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If `from` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidSender`] is returned.
    /// If the `from` is not the caller (`msg::sender()`),
    /// and the caller does not have the right to approve, then the error
    /// [`erc1155::Error::MissingApprovalForAll`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_received`] hasn't returned
    /// its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `to` cannot be the `Address::ZERO`.
    /// * If the caller is not `from`, it must have been approved to spend
    ///   `from`'s tokens via [`IErc1155::set_approval_for_all`].
    /// * `from` must have a balance of tokens of type `id` of at least `value`
    ///   amount.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`erc1155::IERC1155Receiver::on_erc_1155_received`] and return the
    ///   acceptance value.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event.
    ///
    /// # Panics
    ///
    /// Should not panic.
    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        value: U256,
        data: Bytes,
    ) -> Result<(), erc1155::Error> {
        self.erc1155.authorize_transfer(from)?;
        self.do_safe_transfer_from(from, to, vec![id], vec![value], &data)
    }

    /// Batched version of [`Erc1155::safe_transfer_from`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all tokens ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If `from` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidSender`] is returned.
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    /// If the `from` is not the caller (`msg::sender()`),
    /// and the caller does not have the right to approve, then the error
    /// [`erc1155::Error::MissingApprovalForAll`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] hasn't
    /// returned its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements
    ///
    /// * `to` cannot be the `Address::ZERO`.
    /// * If the caller is not `from`, it must have been approved to spend
    ///   `from`'s tokens via [`IErc1155::set_approval_for_all`].
    /// * `from` must have a balance of tokens being transferred of at least
    ///   transferred amount.
    /// * `ids` and `values` must have the same length.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] and return
    ///   the acceptance magic value.
    ///
    /// # Events
    ///
    /// Emits either a [`TransferSingle`] or a [`erc1155::TransferBatch`] event,
    /// depending on the length of the array arguments.
    ///
    /// # Panics
    ///
    /// Should not panic.
    pub fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: Bytes,
    ) -> Result<(), erc1155::Error> {
        self.erc1155.authorize_transfer(from)?;
        self.do_safe_transfer_from(from, to, ids, values, &data)
    }
}

impl Erc1155Supply {
    // Note: overriding `_update` requires reimplementing all of the functions
    // that use it.

    /// Override of [`Erc1155::_update`] that updates the supply of tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_ids` - Array of all token id.
    /// * `values` - Array of all amount of tokens to be supplied.
    ///
    /// # Errors
    ///
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    ///
    /// NOTE: The ERC-1155 acceptance check is not performed in this function.
    /// See [`Self::_update_with_acceptance_check`] instead.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// If updated balance and/or supply exceeds `U256::MAX`, may happen during
    /// the `mint` operation.
    pub fn _update(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), erc1155::Error> {
        self.erc1155._update(from, to, token_ids.clone(), values.clone())?;

        if from.is_zero() {
            let mut total_mint_value = U256::ZERO;
            token_ids.iter().zip(values.iter()).for_each(
                |(&token_id, &value)| {
                    let total_supply =
                        self.total_supply(token_id).checked_add(value).expect(
                            "should not exceed `U256::MAX` for `_total_supply`",
                        );
                    self._total_supply.setter(token_id).set(total_supply);
                    total_mint_value += value;
                },
            );
            let total_supply_all =
                self.total_supply_all().checked_add(total_mint_value).expect(
                    "should not exceed `U256::MAX` for `_total_supply_all`",
                );
            self._total_supply_all.set(total_supply_all);
        }

        if to.is_zero() {
            let mut total_burn_value = U256::ZERO;
            token_ids.iter().zip(values.iter()).for_each(
                |(&token_id, &value)| {
                    /*
                    SAFETY: Overflow not possible:
                    values[i] <= balance_of(from, token_ids[i]) <= total_supply(token_ids[i])
                     */
                    self._total_supply
                        .setter(token_id)
                        .sub_assign_unchecked(value);
                    /*
                    SAFETY: Overflow not possible:
                    sum_i(values[i]) <= sum_i(total_supply(token_ids[i])) <= total_supply_all
                     */
                    total_burn_value += value;
                },
            );
            /*
            SAFETY: Overflow not possible:
            totalBurnValue = sum_i(values[i]) <= sum_i(totalSupply(ids[i])) <= totalSupplyAll
             */
            let total_supply_all =
                self._total_supply_all.get() - total_burn_value;
            self._total_supply_all.set(total_supply_all);
        }
        Ok(())
    }

    /// Version of [`Self::_update`] that performs the token acceptance check by
    /// calling [`erc1155::IERC1155Receiver::on_erc_1155_received`] or
    /// [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] on the
    /// receiver address if it contains code.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_received`] hasn't returned
    /// its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] hasn't
    /// returned its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// If updated balance and/or supply exceeds `U256::MAX`, may happen during
    /// the `mint` operation.
    pub fn _update_with_acceptance_check(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
    ) -> Result<(), erc1155::Error> {
        self._update(from, to, ids.clone(), values.clone())?;

        if !to.is_zero() {
            self.erc1155._check_on_erc1155_received(
                msg::sender(),
                from,
                to,
                erc1155::Erc1155ReceiverData::new(ids, values),
                data.to_vec().into(),
            )?;
        }

        Ok(())
    }

    /// Creates a `value` amount of tokens of type `id`, and assigns
    /// them to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `id` - Token id.
    /// * `value` - Amount of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If [`IERC1155Receiver::on_erc_1155_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event.
    ///
    /// # Panics
    ///
    /// If updated balance and/or supply exceeds `U256::MAX`.
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        value: U256,
        data: &Bytes,
    ) -> Result<(), erc1155::Error> {
        self._do_mint(to, vec![id], vec![value], data)
    }

    /// Batched version of [`Self::_mint`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all tokens ids to be minted.
    /// * `values` - Array of all amounts of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_received`] hasn't returned
    /// its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] hasn't
    /// returned its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// If updated balance and/or supply exceeds `U256::MAX`.
    pub fn _mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
    ) -> Result<(), erc1155::Error> {
        self._do_mint(to, ids, values, data)
    }

    /// Destroys a `value` amount of tokens of type `id` from `from`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `id` - Token id to be burnt.
    /// * `value` - Amount of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// If `from` is the `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidSender`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event.
    ///
    /// # Panics
    ///
    /// Should not panic.
    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        value: U256,
    ) -> Result<(), erc1155::Error> {
        self._do_burn(from, vec![id], vec![value])
    }

    /// Batched version of [`Self::_burn`].
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `ids` - Array of all tokens ids to be burnt.
    /// * `values` - Array of all amounts of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// If `from` is the `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidSender`] is returned.
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// Should not panic.
    pub fn _burn_batch(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), erc1155::Error> {
        self._do_burn(from, ids, values)
    }
}

impl Erc1155Supply {
    /// Creates `values` of tokens specified by `ids`, and assigns
    /// them to `to`. Performs the token acceptance check by
    /// calling [`erc1155::IERC1155Receiver::on_erc_1155_received`] or
    /// [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] on the `to`
    /// address if it contains code.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids to be minted.
    /// * `values` - Array of all amounts of tokens to be minted.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_received`] hasn't returned
    /// its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] hasn't
    /// returned its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// If updated balance and/or supply exceeds `U256::MAX`.
    fn _do_mint(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
    ) -> Result<(), erc1155::Error> {
        if to.is_zero() {
            return Err(erc1155::Error::InvalidReceiver(
                erc1155::ERC1155InvalidReceiver { receiver: to },
            ));
        }
        self._update_with_acceptance_check(
            Address::ZERO,
            to,
            ids,
            values,
            data,
        )?;
        Ok(())
    }

    /// Destroys `values` amounts of tokens specified by `ids` from `from`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to burn tokens from.
    /// * `ids` - Array of all token ids to be burnt.
    /// * `values` - Array of all amount of tokens to be burnt.
    ///
    /// # Errors
    ///
    /// If `from` is the `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidSender`] is returned.
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// Should not panic.
    fn _do_burn(
        &mut self,
        from: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), erc1155::Error> {
        if from.is_zero() {
            return Err(erc1155::Error::InvalidSender(
                erc1155::ERC1155InvalidSender { sender: from },
            ));
        }
        self._update_with_acceptance_check(
            from,
            Address::ZERO,
            ids,
            values,
            &vec![].into(),
        )?;
        Ok(())
    }

    /// Transfers `values` of tokens specified by `ids` from `from` to `to`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account of the recipient.
    /// * `ids` - Array of all token ids.
    /// * `values` - Array of all amount of tokens to be transferred.
    /// * `data` - Additional data with no specified format, sent in call to
    ///   `to`.
    ///
    /// # Errors
    ///
    /// If `to` is the `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If `from` is the `Address::ZERO`, then the error
    /// [`erc1155::Error::InvalidSender`] is returned.
    /// If length of `ids` is not equal to length of `values`, then the
    /// error [`erc1155::Error::InvalidArrayLength`] is returned.
    /// If `value` is greater than the balance of the `from` account,
    /// then the error [`erc1155::Error::InsufficientBalance`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_received`] hasn't returned
    /// its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    /// If [`erc1155::IERC1155Receiver::on_erc_1155_batch_received`] hasn't
    /// returned its interface id or returned with error, then the error
    /// [`erc1155::Error::InvalidReceiver`] is returned.
    ///
    /// # Events
    ///
    /// Emits a [`erc1155::TransferSingle`] event if the arrays contain one
    /// element, and [`erc1155::TransferBatch`] otherwise.
    ///
    /// # Panics
    ///
    /// If updated balance and/or supply exceeds `U256::MAX`.
    fn do_safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        values: Vec<U256>,
        data: &Bytes,
    ) -> Result<(), erc1155::Error> {
        if to.is_zero() {
            return Err(erc1155::Error::InvalidReceiver(
                erc1155::ERC1155InvalidReceiver { receiver: to },
            ));
        }
        if from.is_zero() {
            return Err(erc1155::Error::InvalidSender(
                erc1155::ERC1155InvalidSender { sender: from },
            ));
        }
        self._update_with_acceptance_check(from, to, ids, values, data)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};

    use super::Erc1155Supply;
    use crate::token::erc1155::IErc1155;

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u32>())).collect()
    }

    pub(crate) fn random_values(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
    }

    fn setup(
        contract: &mut Erc1155Supply,
        receiver: Address,
        size: usize,
    ) -> (Vec<U256>, Vec<U256>) {
        let token_ids = random_token_ids(size);
        let values = random_values(size);

        contract
            ._mint_batch(
                receiver,
                token_ids.clone(),
                values.clone(),
                &vec![].into(),
            )
            .expect("should mint");
        (token_ids, values)
    }

    #[motsu::test]
    fn before_mint(contract: Erc1155Supply) {
        let token_id = random_token_ids(1)[0];
        assert_eq!(U256::ZERO, contract.total_supply(token_id));
        assert_eq!(U256::ZERO, contract.total_supply_all());
        assert!(!contract.exists(token_id));
    }

    #[motsu::test]
    fn after_mint_single(contract: Erc1155Supply) {
        let (token_ids, values) = setup(contract, ALICE, 1);
        assert_eq!(values[0], contract.total_supply(token_ids[0]));
        assert_eq!(values[0], contract.total_supply_all());
        assert!(contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn after_mint_batch(contract: Erc1155Supply) {
        let (token_ids, values) = setup(contract, ALICE, 4);
        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            assert_eq!(value, contract.erc1155.balance_of(ALICE, token_id));
            assert!(contract.exists(token_id));
        }
        let total_supply: U256 = values.iter().sum();
        assert_eq!(total_supply, contract.total_supply_all());
    }

    #[motsu::test]
    fn after_burn_single(contract: Erc1155Supply) {
        let (token_ids, values) = setup(contract, ALICE, 1);
        contract._burn(ALICE, token_ids[0], values[0]).expect("should burn");

        assert_eq!(U256::ZERO, contract.total_supply(token_ids[0]));
        assert_eq!(U256::ZERO, contract.total_supply_all());
        assert!(!contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn after_burn_batch(contract: Erc1155Supply) {
        let (token_ids, values) = setup(contract, ALICE, 4);
        contract
            ._burn_batch(ALICE, token_ids.clone(), values.clone())
            .expect("should burn batch");

        for &token_id in token_ids.iter() {
            assert_eq!(
                U256::ZERO,
                contract.erc1155.balance_of(ALICE, token_id)
            );
            assert!(!contract.exists(token_id));
        }
        assert_eq!(U256::ZERO, contract.total_supply_all());
    }

    #[motsu::test]
    fn supply_unaffected_by_no_op(contract: Erc1155Supply) {
        let token_ids = random_token_ids(1);
        let values = random_values(1);

        contract
            ._update(Address::ZERO, Address::ZERO, token_ids.clone(), values)
            .expect("should supply");
        assert_eq!(U256::ZERO, contract.total_supply(token_ids[0]));
        assert_eq!(U256::ZERO, contract.total_supply_all());
        assert!(!contract.exists(token_ids[0]));
    }
}
