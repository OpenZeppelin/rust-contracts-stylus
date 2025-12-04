//! Optional Burnable extension of the ERC-1155 standard.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::msg;

use crate::token::erc1155::{
    self, ERC1155MissingApprovalForAll, Erc1155, IErc1155,
};

/// Extension of [`Erc1155`] that allows token holders to destroy both their
/// own tokens and those that they have been approved to use.
#[interface_id]
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
    /// * [`erc1155::Error::InvalidSender`] - If `from` is the
    ///   [`Address::ZERO`].
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
    /// * [`erc1155::Error::InvalidSender`] - If `from` is the
    ///   [`Address::ZERO`].
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

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};
    use motsu::prelude::*;

    use super::IErc1155Burnable;
    use crate::token::erc1155::{
        tests::{random_token_ids, random_values},
        ERC1155InsufficientBalance, ERC1155InvalidSender,
        ERC1155MissingApprovalForAll, Erc1155, Error, IErc1155,
    };

    trait Init {
        fn init(
            &mut self,
            receiver: Address,
            size: usize,
        ) -> (Vec<U256>, Vec<U256>);
    }

    impl Init for Erc1155 {
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
                &vec![0, 1, 2, 3].into(),
            )
            .expect("Mint failed");
            (token_ids, values)
        }
    }

    #[motsu::test]
    fn burns(contract: Contract<Erc1155>, alice: Address) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);

        let initial_balance =
            contract.sender(alice).balance_of(alice, token_ids[0]);
        assert_eq!(values[0], initial_balance);

        contract
            .sender(alice)
            .burn(alice, token_ids[0], values[0])
            .motsu_expect("should burn own tokens");

        let balance = contract.sender(alice).balance_of(alice, token_ids[0]);
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn burns_with_approval(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);

        let initial_balance =
            contract.sender(alice).balance_of(bob, token_ids[0]);
        assert_eq!(values[0], initial_balance);

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        contract
            .sender(alice)
            .burn(bob, token_ids[0], values[0])
            .motsu_expect("should burn Bob's token");

        let balance = contract.sender(alice).balance_of(bob, token_ids[0]);
        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn error_when_missing_approval_burns(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 1);

        let err = contract
            .sender(alice)
            .burn(bob, token_ids[0], values[0])
            .motsu_expect_err("should not burn tokens without approval");

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                owner,
                operator
            }) if owner == bob && operator == alice
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_burns(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);
        let invalid_sender = Address::ZERO;

        contract
            .sender(invalid_sender)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        let err = contract
            .sender(alice)
            .burn(invalid_sender, token_ids[0], values[0])
            .motsu_expect_err(
                "should not burn tokens from the `Address::ZERO`",
            );

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender,
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn error_when_insufficient_balance_burn(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 1);

        let token_id = token_ids[0];
        let value = values[0];
        let to_burn = value + U256::ONE;

        let err = contract
            .sender(alice)
            .burn(alice, token_id, to_burn)
            .motsu_expect_err("should return `ERC1155InsufficientBalance`");

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
    fn burns_batch(contract: Contract<Erc1155>, alice: Address) {
        let (token_ids, values) = contract.sender(alice).init(alice, 4);

        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            let balance = contract.sender(alice).balance_of(alice, token_id);
            assert_eq!(value, balance);
        }

        contract
            .sender(alice)
            .burn_batch(alice, token_ids.clone(), values.clone())
            .motsu_expect("should burn own tokens in batch");

        for token_id in token_ids {
            let balance = contract.sender(alice).balance_of(alice, token_id);
            assert_eq!(U256::ZERO, balance);
        }
    }

    #[motsu::test]
    fn burns_batch_with_approval(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 4);

        for (&token_id, &value) in token_ids.iter().zip(values.iter()) {
            let balance = contract.sender(alice).balance_of(bob, token_id);
            assert_eq!(value, balance);
        }

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        contract
            .sender(alice)
            .burn_batch(bob, token_ids.clone(), values.clone())
            .motsu_expect("should burn Bob's tokens in batch");

        for token_id in token_ids {
            let balance = contract.sender(alice).balance_of(bob, token_id);
            assert_eq!(U256::ZERO, balance);
        }
    }

    #[motsu::test]
    fn error_when_missing_approval_burn_batch(
        contract: Contract<Erc1155>,
        alice: Address,
        bob: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(bob, 2);

        let err = contract
            .sender(alice)
            .burn_batch(bob, token_ids.clone(), values.clone())
            .motsu_expect_err(
                "should not burn tokens in batch without approval",
            );

        assert!(matches!(
            err,
            Error::MissingApprovalForAll(ERC1155MissingApprovalForAll {
                owner,
                operator
            }) if owner == bob && operator == alice
        ));
    }

    #[motsu::test]
    fn error_when_invalid_sender_burn_batch(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 5);
        let invalid_sender = Address::ZERO;

        contract
            .sender(invalid_sender)
            .set_approval_for_all(alice, true)
            .motsu_expect("should approve Bob's tokens to Alice");

        let err = contract
            .sender(alice)
            .burn_batch(invalid_sender, token_ids.clone(), values.clone())
            .motsu_expect_err(
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
    fn error_when_insufficient_balance_burn_batch(
        contract: Contract<Erc1155>,
        alice: Address,
    ) {
        let (token_ids, values) = contract.sender(alice).init(alice, 5);
        let to_burn: Vec<U256> = values.iter().map(|v| v + U256::ONE).collect();

        let err = contract
            .sender(alice)
            .burn_batch(alice, token_ids.clone(), to_burn.clone())
            .motsu_expect_err("should return `ERC1155InsufficientBalance`");

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
