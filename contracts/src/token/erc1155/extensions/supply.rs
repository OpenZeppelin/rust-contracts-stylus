//! Extension of ERC-1155 that adds tracking of total supply per token id.
//!
//! Useful for scenarios where Fungible and Non-fungible tokens have to be
//! clearly identified. Note: While a `total_supply` of 1 might mean the
//! corresponding is an NFT, there are no guarantees that no other tokens
//! with the same id are not going to be minted.
//!
//! NOTE: This contract implies a global limit of 2**256 - 1 to the number
//! of tokens that can be minted.
//!
//! CAUTION: This extension should not be added in an upgrade to an already
//! deployed contract.

use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    abi::Bytes,
    msg,
    prelude::*,
    storage::{StorageMap, StorageU256},
};

use crate::{
    token::erc1155::{self, Erc1155, Error, IErc1155},
    utils::{
        introspection::erc165::IErc165,
        math::storage::{AddAssignChecked, SubAssignUnchecked},
    },
};

/// State of an [`Erc1155Supply`] contract.
#[storage]
pub struct Erc1155Supply {
    /// [`Erc1155`] contract.
    pub erc1155: Erc1155,
    /// Mapping from token id to total supply.
    pub(crate) total_supply: StorageMap<U256, StorageU256>,
    /// Total supply of all token ids.
    pub(crate) total_supply_all: StorageU256,
}

/// Required interface of a [`Erc1155Supply`] contract.
#[interface_id]
pub trait IErc1155Supply: IErc165 {
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

#[public]
#[implements(IErc1155<Error = Error>, IErc1155Supply, IErc165)]
impl Erc1155Supply {}

#[public]
impl IErc1155Supply for Erc1155Supply {
    fn total_supply(&self, id: U256) -> U256 {
        self.total_supply.get(id)
    }

    fn total_supply_all(&self) -> U256 {
        self.total_supply_all.get()
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
    ) -> Result<Vec<U256>, Self::Error> {
        self.erc1155.balance_of_batch(accounts, ids)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Self::Error> {
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
    ) -> Result<(), Self::Error> {
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
    ) -> Result<(), Self::Error> {
        self.erc1155.authorize_transfer(from)?;
        self.do_safe_transfer_from(from, to, ids, values, &data)
    }
}

#[public]
impl IErc165 for Erc1155Supply {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc1155Supply>::interface_id() == interface_id
            || self.erc1155.supports_interface(interface_id)
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
    /// * If updated balance and/or supply exceeds [`U256::MAX`], may happen
    ///   during the [`Self::_mint`] operation.
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
                self.total_supply.setter(token_id).add_assign_checked(
                    value,
                    "should not exceed `U256::MAX` for `total_supply`",
                );
            }

            let total_mint_value = values.iter().sum();
            self.total_supply_all.add_assign_checked(
                total_mint_value,
                "should not exceed `U256::MAX` for `total_supply_all`",
            );
        }

        if to.is_zero() {
            for (token_id, &value) in token_ids.into_iter().zip(values.iter()) {
                /*
                 * SAFETY: Overflow not possible:
                 * values[i] <= balance_of(from, token_ids[i]) <=
                 * total_supply(token_ids[i])
                 */
                self.total_supply.setter(token_id).sub_assign_unchecked(value);
            }

            let total_burn_value: U256 = values.into_iter().sum();
            /*
             * SAFETY: Overflow not possible:
             * total_burn_value = sum_i(values[i]) <=
             * sum_i(total_supply(ids[i])) <= total_supply_all
             */
            self.total_supply_all.sub_assign_unchecked(total_burn_value);
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

#[cfg(test)]
mod tests {
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{uint, Address, U256},
        prelude::*,
    };

    use super::*;
    use crate::{
        token::erc1155::{
            receiver::tests::{
                BadSelectorReceiver1155, EmptyReasonReceiver1155,
                RevertingReceiver1155,
            },
            tests::{random_token_ids, random_values},
            ERC1155InvalidArrayLength, ERC1155InvalidReceiver,
            ERC1155InvalidSender, InvalidReceiverWithReason,
        },
        utils::introspection::erc165::IErc165,
    };

    unsafe impl TopLevelStorage for Erc1155Supply {}

    impl Erc1155Supply {
        fn init(
            &mut self,
            receiver: Address,
            size: usize,
        ) -> (Vec<U256>, Vec<U256>) {
            let token_ids = random_token_ids(size);
            let values = random_values(size);

            self._mint_batch(
                receiver,
                token_ids.clone(),
                values.clone(),
                &vec![].into(),
            )
            .motsu_expect("should mint");
            (token_ids, values)
        }
    }

    #[motsu::test]
    fn before_mint(contract: Contract<Erc1155Supply>, alice: Address) {
        let token_id = random_token_ids(1)[0];
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(token_id));
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply_all());
        assert!(!contract.sender(alice).exists(token_id));
    }

    #[motsu::test]
    fn after_mint_single(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);
        assert_eq!(
            values[0],
            contract.sender(alice).balance_of(bob, token_ids[0])
        );
        assert_eq!(
            values[0],
            contract.sender(alice).total_supply(token_ids[0])
        );
        assert_eq!(values[0], contract.sender(alice).total_supply_all());
        assert!(contract.sender(alice).exists(token_ids[0]));
    }

    #[motsu::test]
    fn after_mint_batch(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 4);
        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            assert_eq!(value, contract.sender(alice).balance_of(bob, token_id));
            assert_eq!(value, contract.sender(alice).total_supply(token_id));
            assert!(contract.sender(alice).exists(token_id));
        }
        let total_supply_all: U256 = values.iter().sum();
        assert_eq!(total_supply_all, contract.sender(alice).total_supply_all());
    }

    #[motsu::test]
    fn mint_reverts_on_invalid_receiver(
        contract: Contract<Erc1155Supply>,
        alice: Address,
    ) {
        let token_id = random_token_ids(1)[0];
        let two = uint!(2_U256);
        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            ._mint(invalid_receiver, token_id, two, &vec![].into())
            .motsu_expect_err("should revert with `InvalidReceiver`");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver {
                receiver
            }) if receiver == invalid_receiver
        ));
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `total_supply`"]
    fn mint_panics_on_total_supply_overflow(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        let token_id = random_token_ids(1)[0];
        let two = uint!(2_U256);
        let three = uint!(3_U256);
        contract
            .sender(alice)
            ._mint(bob, token_id, U256::MAX / two, &vec![].into())
            .motsu_expect("should mint to bob");
        contract
            .sender(alice)
            ._mint(dave, token_id, U256::MAX / two, &vec![].into())
            .motsu_expect("should mint to dave");
        // This should panic.
        _ = contract.sender(alice)._mint(bob, token_id, three, &vec![].into());
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `total_supply_all`"]
    fn mint_panics_on_total_supply_all_overflow(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let token_ids = random_token_ids(2);
        contract
            .sender(alice)
            ._mint(bob, token_ids[0], U256::MAX, &vec![].into())
            .motsu_expect("should mint");
        // This should panic.
        _ = contract.sender(alice)._mint(
            bob,
            token_ids[1],
            U256::ONE,
            &vec![].into(),
        );
    }

    #[motsu::test]
    fn after_burn_single(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);
        contract
            .sender(alice)
            ._burn(bob, token_ids[0], values[0])
            .motsu_expect("should burn");

        assert_eq!(
            U256::ZERO,
            contract.sender(alice).total_supply(token_ids[0])
        );
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply_all());
        assert!(!contract.sender(alice).exists(token_ids[0]));
    }

    #[motsu::test]
    fn after_burn_batch(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 4);
        contract
            .sender(alice)
            ._burn_batch(bob, token_ids.clone(), values.clone())
            .motsu_expect("should burn batch");

        for &token_id in &token_ids {
            assert_eq!(
                U256::ZERO,
                contract.sender(alice).balance_of(bob, token_id)
            );
            assert!(!contract.sender(alice).exists(token_id));
            assert_eq!(
                U256::ZERO,
                contract.sender(alice).total_supply(token_id)
            );
        }
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply_all());
    }

    #[motsu::test]
    fn burn_reverts_when_invalid_sender(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);
        let invalid_sender = Address::ZERO;

        let err = contract
            .sender(alice)
            ._burn(invalid_sender, token_ids[0], values[0])
            .motsu_expect_err("should not burn token for invalid sender");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn supply_unaffected_by_no_op(
        contract: Contract<Erc1155Supply>,
        alice: Address,
    ) {
        let token_ids = random_token_ids(1);
        let values = random_values(1);

        contract
            .sender(alice)
            ._update(Address::ZERO, Address::ZERO, token_ids.clone(), values)
            .motsu_expect("should supply");
        assert_eq!(
            U256::ZERO,
            contract.sender(alice).total_supply(token_ids[0])
        );
        assert_eq!(U256::ZERO, contract.sender(alice).total_supply_all());
        assert!(!contract.sender(alice).exists(token_ids[0]));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc1155Supply as IErc1155Supply>::interface_id();
        let expected: B32 = 0xeac6339d_u32.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc1155Supply>, alice: Address) {
        assert!(contract.sender(alice).supports_interface(
            <Erc1155Supply as IErc1155Supply>::interface_id()
        ));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc1155Supply as IErc165>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc1155Supply as IErc1155>::interface_id()));

        let fake_interface_id: B32 = 0x12345678_u32.into();
        assert!(!contract.sender(alice).supports_interface(fake_interface_id));
    }

    // ---------------- Additional tests for full coverage -----------------

    #[motsu::test]
    fn balance_of_batch_works(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        // Mint 3 token ids to bob
        let (ids, values) = contract.sender(alice).init(bob, 3);
        let accounts = vec![bob, bob, bob];
        let balances = contract
            .sender(alice)
            .balance_of_batch(accounts, ids.clone())
            .motsu_expect("should get balances");
        assert_eq!(balances, values);
    }

    #[motsu::test]
    fn balance_of_batch_invalid_length(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (ids, _values) = contract.sender(alice).init(bob, 2);
        let accounts = vec![bob]; // mismatch lengths (1 vs 2)
        let err = contract
            .sender(alice)
            .balance_of_batch(accounts.clone(), ids.clone())
            .motsu_expect_err("should fail on array length mismatch");
        // just ensure it is the ERC1155 error enum variant
        assert!(
            matches!(err, Error::InvalidArrayLength(ERC1155InvalidArrayLength { ids_length, values_length }) if ids_length == U256::from(ids.len()) && values_length == U256::from(accounts.len()))
        );
    }

    #[motsu::test]
    fn approval_set_and_query(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        // Initially not approved
        assert!(!contract.sender(alice).is_approved_for_all(alice, bob));

        // Approve bob
        contract
            .sender(alice)
            .set_approval_for_all(bob, true)
            .motsu_expect("should approve");
        assert!(contract.sender(alice).is_approved_for_all(alice, bob));

        // Revoke
        contract
            .sender(alice)
            .set_approval_for_all(bob, false)
            .motsu_expect("should revoke");
        assert!(!contract.sender(alice).is_approved_for_all(alice, bob));
    }

    #[motsu::test]
    fn approval_reverts_on_zero_operator(
        contract: Contract<Erc1155Supply>,
        alice: Address,
    ) {
        let zero = Address::ZERO;
        let err = contract
            .sender(alice)
            .set_approval_for_all(zero, true)
            .motsu_expect_err("should fail for zero operator");
        assert!(matches!(
            err,
            Error::InvalidOperator(crate::token::erc1155::ERC1155InvalidOperator { operator }) if operator == zero
        ));
    }

    #[motsu::test]
    fn safe_transfer_from_by_owner(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        // Mint to alice
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];

        // Transfer from alice to bob as owner (no approval needed)
        contract
            .sender(alice)
            .safe_transfer_from(alice, bob, id, value, vec![].into())
            .motsu_expect("owner should transfer");

        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice, id));
        assert_eq!(value, contract.sender(alice).balance_of(bob, id));
    }

    #[motsu::test]
    fn safe_transfer_from_by_operator(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        // Mint to alice
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];

        // Alice approves bob
        contract
            .sender(alice)
            .set_approval_for_all(bob, true)
            .motsu_expect("should approve");

        // Bob transfers from alice to dave
        contract
            .sender(bob)
            .safe_transfer_from(alice, dave, id, value, vec![].into())
            .motsu_expect("operator should transfer");

        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice, id));
        assert_eq!(value, contract.sender(alice).balance_of(dave, id));
    }

    #[motsu::test]
    fn safe_transfer_from_missing_approval(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
        dave: Address,
    ) {
        // Mint to alice
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];

        // Bob tries to transfer without approval
        let err = contract
            .sender(bob)
            .safe_transfer_from(alice, dave, id, value, vec![].into())
            .motsu_expect_err("should fail: missing approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(crate::token::erc1155::ERC1155MissingApprovalForAll { operator, owner }) if operator == bob && owner == alice
        ));
    }

    #[motsu::test]
    fn safe_transfer_from_invalid_receiver_zero(
        contract: Contract<Erc1155Supply>,
        alice: Address,
    ) {
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];
        let to = Address::ZERO;
        let err = contract
            .sender(alice)
            .safe_transfer_from(alice, to, id, value, vec![].into())
            .motsu_expect_err("should fail for zero receiver");
        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == to
        ));
    }

    #[motsu::test]
    fn safe_batch_transfer_from_works(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        // Mint 3 ids to alice
        let (ids, values) = contract.sender(alice).init(alice, 3);
        contract
            .sender(alice)
            .safe_batch_transfer_from(
                alice,
                bob,
                ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .motsu_expect("batch transfer should work");

        for (&id, &value) in ids.iter().zip(values.iter()) {
            assert_eq!(
                U256::ZERO,
                contract.sender(alice).balance_of(alice, id)
            );
            assert_eq!(value, contract.sender(alice).balance_of(bob, id));
        }
    }

    #[motsu::test]
    fn safe_batch_transfer_from_invalid_array_length(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        // Mint 2 ids to alice
        let (ids, mut values) = contract.sender(alice).init(alice, 2);
        // Make lengths mismatched by dropping one value
        values.pop();
        let err = contract
            .sender(alice)
            .safe_batch_transfer_from(
                alice,
                bob,
                ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .motsu_expect_err("should fail for array length mismatch");
        assert!(
            matches!(err, Error::InvalidArrayLength(ERC1155InvalidArrayLength { ids_length, values_length }) if ids_length == U256::from(ids.len()) && values_length == U256::from(values.len()))
        );

        drop(contract);

        let contract = Contract::<Erc1155Supply>::from_tag("new_contract");
        let (mut ids, values) = contract.sender(alice).init(alice, 2);
        // Also mismatch ids by adding one more id
        ids.push(uint!(999_U256));

        let err2 = contract
            .sender(alice)
            .safe_batch_transfer_from(
                alice,
                bob,
                ids.clone(),
                values.clone(),
                vec![].into(),
            )
            .motsu_expect_err("should still fail for array length mismatch");
        assert!(
            matches!(err2, Error::InvalidArrayLength(ERC1155InvalidArrayLength { ids_length, values_length }) if ids_length == U256::from(ids.len()) && values_length == U256::from(values.len()))
        );
    }

    // ----------------------- Acceptance-check failures ----------------------

    #[motsu::test]
    fn mint_rejects_when_receiver_returns_wrong_selector(
        contract: Contract<Erc1155Supply>,
        bad: Contract<BadSelectorReceiver1155>,
        alice: Address,
    ) {
        let id = U256::ONE;
        let value = uint!(5_U256);

        let err = contract
            .sender(alice)
            ._mint(bad.address(), id, value, &vec![].into())
            .motsu_expect_err(
                "receiver returning wrong selector must be rejected",
            );

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == bad.address()
        ));
        assert_eq!(
            U256::ZERO,
            contract.sender(alice).balance_of(bad.address(), id)
        );
    }

    #[motsu::test]
    fn mint_bubbles_revert_reason_from_receiver(
        contract: Contract<Erc1155Supply>,
        reverting: Contract<RevertingReceiver1155>,
        alice: Address,
    ) {
        let id = uint!(2_U256);
        let value = uint!(7_U256);

        let err = contract
            .sender(alice)
            ._mint(reverting.address(), id, value, &vec![].into())
            .motsu_expect_err("receiver reverting should bubble reason");

        assert!(matches!(
            err,
            Error::InvalidReceiverWithReason(InvalidReceiverWithReason { reason }) if reason == "Receiver rejected single"
        ));
        assert_eq!(
            U256::ZERO,
            contract.sender(alice).balance_of(reverting.address(), id)
        );
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    #[motsu::test]
    #[ignore = "TODO: un-ignore when https://github.com/OpenZeppelin/stylus-test-helpers/issues/118 is fixed"]
    fn mint_rejects_on_empty_revert_reason(
        contract: Contract<Erc1155Supply>,
        empty: Contract<EmptyReasonReceiver1155>,
        alice: Address,
    ) {
        let id = uint!(3_U256);
        let value = uint!(9_U256);

        let err = contract
            .sender(alice)
            ._mint(empty.address(), id, value, &vec![].into())
            .motsu_expect_err("empty revert must map to InvalidReceiver");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == empty.address()
        ));
    }

    #[motsu::test]
    fn transfer_rejects_when_receiver_returns_wrong_selector(
        contract: Contract<Erc1155Supply>,
        bad: Contract<BadSelectorReceiver1155>,
        alice: Address,
    ) {
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];

        let err = contract
            .sender(alice)
            .safe_transfer_from(alice, bad.address(), id, value, vec![].into())
            .motsu_expect_err("wrong selector should be rejected in transfer");

        assert!(matches!(
            err,
            Error::InvalidReceiver(ERC1155InvalidReceiver { receiver }) if receiver == bad.address()
        ));
        assert_eq!(value, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn transfer_bubbles_revert_reason_from_receiver(
        contract: Contract<Erc1155Supply>,
        reverting: Contract<RevertingReceiver1155>,
        alice: Address,
    ) {
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];

        let err = contract
            .sender(alice)
            .do_safe_transfer_from(
                alice,
                reverting.address(),
                vec![id],
                vec![value],
                &vec![].into(),
            )
            .motsu_expect_err("revert reason should bubble in transfer");

        assert!(matches!(
            err,
            Error::InvalidReceiverWithReason(InvalidReceiverWithReason { reason }) if reason == "Receiver rejected batch" || reason == "Receiver rejected single"
        ));
        assert_eq!(value, contract.sender(alice).balance_of(alice, id));
    }

    // ----------------------- Other internal error paths ----------------------

    #[motsu::test]
    fn burn_reverts_on_insufficient_balance(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (ids, values) = contract.sender(alice).init(bob, 1);
        let token_id = ids[0];
        let balance = values[0];
        let value = balance + U256::ONE;

        let err = contract
            .sender(alice)
            ._burn(bob, token_id, value)
            .motsu_expect_err("should not burn more than balance");

        assert!(matches!(
            err,
            Error::InsufficientBalance(crate::token::erc1155::ERC1155InsufficientBalance { sender, balance: b, needed, token_id: tid })
                if sender == bob && b == balance && needed == value && tid == token_id
        ));
    }

    #[motsu::test]
    fn do_safe_transfer_from_reverts_when_from_is_zero(
        contract: Contract<Erc1155Supply>,
        alice: Address,
        bob: Address,
    ) {
        let (ids, values) = contract.sender(alice).init(alice, 1);
        let id = ids[0];
        let value = values[0];
        let invalid_from = Address::ZERO;

        let err = contract
            .sender(alice)
            .do_safe_transfer_from(
                invalid_from,
                bob,
                vec![id],
                vec![value],
                &vec![].into(),
            )
            .motsu_expect_err("should error when from is zero");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender { sender }) if sender == invalid_from
        ));
    }
}
