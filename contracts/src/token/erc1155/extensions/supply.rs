//! Extension of ERC-1155 that adds tracking of total supply per token id.
//!
//! Useful for scenarios where Fungible and Non-fungible tokens have to be
//! clearly identified. Note: While a `_total_supply` of 1 might mean the
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
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    msg,
    prelude::{public, storage},
    storage::{StorageMap, StorageU256},
};

use crate::{
    token::erc1155::{self, Erc1155, IErc1155},
    utils::math::storage::{AddAssignChecked, SubAssignUnchecked},
};

/// State of an [`Erc1155Supply`] contract.
#[storage]
pub struct Erc1155Supply {
    /// [`Erc1155`] contract.
    pub erc1155: Erc1155,
    /// Mapping from token id to total supply.
    #[allow(clippy::used_underscore_binding)]
    pub _total_supply: StorageMap<U256, StorageU256>,
    /// Total supply of all token ids.
    #[allow(clippy::used_underscore_binding)]
    pub _total_supply_all: StorageU256,
}

/// Required interface of a [`Erc1155Supply`] contract.
#[interface_id]
pub trait IErc1155Supply {
    /// Total value of tokens in with a given id.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    fn total_supply(&self, id: U256) -> U256;

    /// Total value of tokens.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    #[selector(name = "totalSupply")]
    fn total_supply_all(&self) -> U256;

    /// Indicates whether any token exist with a given id, or not.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    fn exists(&self, id: U256) -> bool;
}

impl IErc1155Supply for Erc1155Supply {
    fn total_supply(&self, id: U256) -> U256 {
        self._total_supply.get(id)
    }

    fn total_supply_all(&self) -> U256 {
        *self._total_supply_all
    }

    fn exists(&self, id: U256) -> bool {
        self.total_supply(id) > U256::ZERO
    }
}

#[public]
impl IErc1155 for Erc1155Supply {
    type Error = erc1155::Error;

    fn balance_of(&self, account: Address, id: U256) -> U256 {
        self.erc1155.balance_of(account, id)
    }

    fn balance_of_batch(
        &self,
        accounts: Vec<Address>,
        ids: Vec<U256>,
    ) -> Result<Vec<U256>, erc1155::Error> {
        self.erc1155.balance_of_batch(accounts, ids)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), erc1155::Error> {
        self.erc1155.set_approval_for_all(operator, approved)
    }

    fn is_approved_for_all(&self, account: Address, operator: Address) -> bool {
        self.erc1155.is_approved_for_all(account, operator)
    }

    fn safe_transfer_from(
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

    fn safe_batch_transfer_from(
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
    /// Creates a `value` amount of tokens of type `id`, and assigns
    /// them to `to`.
    ///
    /// Re-export of [`Erc1155::_mint`].
    #[allow(clippy::missing_errors_doc)]
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
    /// Re-export of [`Erc1155::_mint_batch`].
    #[allow(clippy::missing_errors_doc)]
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
    /// Re-export of [`Erc1155::_burn`].
    #[allow(clippy::missing_errors_doc)]
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
    /// Re-export of [`Erc1155::_burn_batch`].
    #[allow(clippy::missing_errors_doc)]
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
    /// Extended version of [`Erc1155::_update`] that updates the supply of
    /// tokens.
    ///
    /// NOTE: The ERC-1155 acceptance check is not performed in this function.
    /// See [`Self::_update_with_acceptance_check`] instead.
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
    /// * [`erc1155::Error::InvalidArrayLength`] - If length of `ids` is not
    ///   equal to length of `values`.
    /// * [`erc1155::Error::InsufficientBalance`] - If `value` is greater than
    ///   the balance of the `from` account.
    ///
    /// # Events
    ///
    /// * [`erc1155::TransferSingle`] - If the arrays contain one element.
    /// * [`erc1155::TransferBatch`] - If the arrays contain more than one
    ///   element.
    ///
    /// # Panics
    ///
    /// * If updated balance and/or supply exceeds `U256::MAX`, may happen
    ///   during the `mint` operation.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), erc1155::Error> {
        self.erc1155._update(from, to, token_ids.clone(), values.clone())?;

        if from.is_zero() {
            for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
                self._total_supply.setter(token_id).add_assign_checked(
                    value,
                    "should not exceed `U256::MAX` for `_total_supply`",
                );
            }

            let total_mint_value = values.iter().sum();
            self._total_supply_all.add_assign_checked(
                total_mint_value,
                "should not exceed `U256::MAX` for `_total_supply_all`",
            );
        }

        if to.is_zero() {
            for (token_id, &value) in token_ids.into_iter().zip(values.iter()) {
                /*
                 * SAFETY: Overflow not possible:
                 * values[i] <= balance_of(from, token_ids[i]) <=
                 * total_supply(token_ids[i])
                 */
                self._total_supply.setter(token_id).sub_assign_unchecked(value);
            }

            let total_burn_value: U256 = values.into_iter().sum();
            /*
             * SAFETY: Overflow not possible:
             * total_burn_value = sum_i(values[i]) <=
             * sum_i(total_supply(ids[i])) <= total_supply_all
             */
            self._total_supply_all.sub_assign_unchecked(total_burn_value);
        }
        Ok(())
    }

    fn _update_with_acceptance_check(
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

    use super::{Erc1155Supply, IErc1155Supply};
    use crate::token::erc1155::{
        tests::{random_token_ids, random_values},
        ERC1155InvalidReceiver, ERC1155InvalidSender, Error, IErc1155,
    };

    const ALICE: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const BOB: Address = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    fn init(
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
        let (token_ids, values) = init(contract, ALICE, 1);
        assert_eq!(values[0], contract.balance_of(ALICE, token_ids[0]));
        assert_eq!(values[0], contract.total_supply(token_ids[0]));
        assert_eq!(values[0], contract.total_supply_all());
        assert!(contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn after_mint_batch(contract: Erc1155Supply) {
        let (token_ids, values) = init(contract, ALICE, 4);
        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            assert_eq!(value, contract.balance_of(ALICE, token_id));
            assert_eq!(value, contract.total_supply(token_id));
            assert!(contract.exists(token_id));
        }
        let total_supply_all: U256 = values.iter().sum();
        assert_eq!(total_supply_all, contract.total_supply_all());
    }

    #[motsu::test]
    fn mint_reverts_on_invalid_receiver(contract: Erc1155Supply) {
        let token_id = random_token_ids(1)[0];
        let two = U256::from(2);
        let invalid_receiver = Address::ZERO;

        let err = contract
            ._mint(invalid_receiver, token_id, two, &vec![].into())
            .expect_err("should revert with `InvalidReceiver`");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `_total_supply`"]
    fn mint_panics_on_total_supply_overflow(contract: Erc1155Supply) {
        let token_id = random_token_ids(1)[0];
        let two = U256::from(2);
        let three = U256::from(3);
        contract
            ._mint(ALICE, token_id, U256::MAX / two, &vec![].into())
            .expect("should mint to ALICE");
        contract
            ._mint(BOB, token_id, U256::MAX / two, &vec![].into())
            .expect("should mint to BOB");
        let _ = contract._mint(ALICE, token_id, three, &vec![].into());
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `_total_supply_all`"]
    fn mint_panics_on_total_supply_all_overflow(contract: Erc1155Supply) {
        let token_ids = random_token_ids(2);
        contract
            ._mint(ALICE, token_ids[0], U256::MAX, &vec![].into())
            .expect("should mint");
        let _ =
            contract._mint(ALICE, token_ids[1], U256::from(1), &vec![].into());
    }

    #[motsu::test]
    fn after_burn_single(contract: Erc1155Supply) {
        let (token_ids, values) = init(contract, ALICE, 1);
        contract._burn(ALICE, token_ids[0], values[0]).expect("should burn");

        assert_eq!(U256::ZERO, contract.total_supply(token_ids[0]));
        assert_eq!(U256::ZERO, contract.total_supply_all());
        assert!(!contract.exists(token_ids[0]));
    }

    #[motsu::test]
    fn after_burn_batch(contract: Erc1155Supply) {
        let (token_ids, values) = init(contract, ALICE, 4);
        contract
            ._burn_batch(ALICE, token_ids.clone(), values.clone())
            .expect("should burn batch");

        for &token_id in &token_ids {
            assert_eq!(
                U256::ZERO,
                contract.erc1155.balance_of(ALICE, token_id)
            );
            assert!(!contract.exists(token_id));
            assert_eq!(U256::ZERO, contract.total_supply(token_id));
        }
        assert_eq!(U256::ZERO, contract.total_supply_all());
    }

    #[motsu::test]
    fn burn_reverts_when_invalid_sender(contract: Erc1155Supply) {
        let (token_ids, values) = init(contract, ALICE, 1);
        let invalid_sender = Address::ZERO;

        let err = contract
            ._burn(invalid_sender, token_ids[0], values[0])
            .expect_err("should not burn token for invalid sender");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
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
