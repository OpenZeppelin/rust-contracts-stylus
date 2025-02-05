//! Optional Burnable extension of the ERC-1155 standard.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::token::erc1155::{
    self, ERC1155MissingApprovalForAll, Erc1155, IErc1155,
};

/// Extension of [`Erc1155`] that allows token holders to destroy both their
/// own tokens and those that they have been approved to use.
pub trait IErc1155Burnable {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Destroys a `value` amount of token from `account`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Account to burn tokens from.
    /// * `token_id` - Token id to be burnt.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`erc1155::Error::MissingApprovalForAll`] - If the caller is not
    ///   `account` address and the `account` has not been approved.
    /// * [`erc1155::Error::InvalidSender`] - If `from` is the `Address::ZERO`.
    /// * [`erc1155::Error::InsufficientBalance`] - If `value` is greater than
    ///   the balance of the `from` account.
    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// Destroys a batch of tokens from `account`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `account` - Accounts to burn tokens from.
    /// * `token_ids` - All token id to be burnt.
    /// * `values` - All amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`erc1155::Error::MissingApprovalForAll`] - If the caller is not
    ///   `account` address and the `account` has not been approved.
    /// * [`erc1155::Error::InvalidSender`] - If `from` is the `Address::ZERO`.
    /// * [`erc1155::Error::InvalidArrayLength`] - If length of `ids` is not
    ///   equal to length of `values`.
    /// * [`erc1155::Error::InsufficientBalance`] - If any of the `values` is
    ///   greater than the balance of the respective token from `tokens` of the
    ///   `from` account.
    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Self::Error>;
}

impl IErc1155Burnable for Erc1155 {
    type Error = erc1155::Error;

    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Self::Error> {
        self.ensure_approved_or_owner(account)?;
        self._burn(account, token_id, value)
    }

    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Self::Error> {
        self.ensure_approved_or_owner(account)?;
        self._burn_batch(account, token_ids, values)
    }
}

impl Erc1155 {
    fn ensure_approved_or_owner(
        &self,
        account: Address,
    ) -> Result<(), erc1155::Error> {
        let sender = msg::sender();
        if account != sender && !self.is_approved_for_all(account, sender) {
            return Err(erc1155::Error::MissingApprovalForAll(
                ERC1155MissingApprovalForAll {
                    owner: account,
                    operator: sender,
                },
            ));
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::msg;

    use super::IErc1155Burnable;
    use crate::token::erc1155::{
        tests::{random_token_ids, random_values},
        ERC1155InsufficientBalance, ERC1155InvalidSender,
        ERC1155MissingApprovalForAll, Erc1155, Error, IErc1155,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    fn init(
        contract: &mut Erc1155,
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
                &vec![0, 1, 2, 3].into(),
            )
            .expect("Mint failed");
        (token_ids, values)
    }

    #[motsu::test]
    fn burns(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 1);

        let initial_balance = contract.balance_of(alice, token_ids[0]);
        assert_eq!(values[0], initial_balance);

        contract
            .burn(alice, token_ids[0], values[0])
            .expect("should burn own tokens");

        let balance = contract.balance_of(alice, token_ids[0]);
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn burns_with_approval(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 1);

        let initial_balance = contract.balance_of(BOB, token_ids[0]);
        assert_eq!(values[0], initial_balance);

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        contract
            .burn(BOB, token_ids[0], values[0])
            .expect("should burn Bob's token");

        let balance = contract.balance_of(BOB, token_ids[0]);
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn error_when_missing_approval_burns(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 1);

        let err = contract
            .burn(BOB, token_ids[0], values[0])
            .expect_err("should not burn tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                owner,
                operator
            }) if owner == BOB && operator == alice
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_burns(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 1);
        let invalid_sender = Address::ZERO;

        contract
            ._operator_approvals
            .setter(invalid_sender)
            .setter(alice)
            .set(true);

        let err = contract
            .burn(invalid_sender, token_ids[0], values[0])
            .expect_err("should not burn tokens from the `Address::ZERO`");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender,
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_burn(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 1);

        let token_id = token_ids[0];
        let value = values[0];
        let to_burn = value + U256::from(1);

        let err = contract
            .burn(alice, token_id, to_burn)
            .expect_err("should return `ERC1155InsufficientBalance`");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id,
            }) if sender == alice && balance == value && needed == to_burn && token_id == token_id
        ));
    }

    #[motsu::test]
    fn burns_batch(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 4);

        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            let balance = contract.balance_of(alice, token_id);
            assert_eq!(value, balance);
        }

        contract
            .burn_batch(alice, token_ids.clone(), values.clone())
            .expect("should burn own tokens in batch");

        for token_id in token_ids {
            let balance = contract.balance_of(alice, token_id);
            assert_eq!(U256::ZERO, balance);
        }
    }

    #[motsu::test]
    fn burns_batch_with_approval(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 4);

        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            let balance = contract.balance_of(BOB, token_id);
            assert_eq!(value, balance);
        }

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        contract
            .burn_batch(BOB, token_ids.clone(), values.clone())
            .expect("should burn Bob's tokens in batch");

        for token_id in token_ids {
            let balance = contract.balance_of(BOB, token_id);
            assert_eq!(U256::ZERO, balance);
        }
    }

    #[motsu::test]
    fn error_when_missing_approval_burn_batch(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 2);

        let err = contract
            .burn_batch(BOB, token_ids.clone(), values.clone())
            .expect_err("should not burn tokens in batch without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                owner,
                operator
            }) if owner == BOB && operator == alice
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_burn_batch(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 5);
        let invalid_sender = Address::ZERO;

        contract
            ._operator_approvals
            .setter(invalid_sender)
            .setter(alice)
            .set(true);

        let err = contract
            .burn_batch(invalid_sender, token_ids.clone(), values.clone())
            .expect_err(
                "should not burn tokens in batch from the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender,
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_burn_batch(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, alice, 5);
        let to_burn: Vec<U256> =
            values.iter().map(|v| v + U256::from(1)).collect();

        let err = contract
            .burn_batch(alice, token_ids.clone(), to_burn.clone())
            .expect_err("should return `ERC1155InsufficientBalance`");

        assert!(matches!(
            err,
            Error::InsufficientBalance(ERC1155InsufficientBalance {
                sender,
                balance,
                needed,
                token_id,
            }) if sender == alice && balance == values[0] && needed == to_burn[0] && token_id == token_ids[0]
        ));
    }
}
