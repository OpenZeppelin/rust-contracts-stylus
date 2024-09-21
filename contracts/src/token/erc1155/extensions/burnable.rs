//! Optional Burnable extension of the ERC-1155 standard.

use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::token::erc1155::{
    ERC1155MissingApprovalForAll, Erc1155, Error, IErc1155,
};

/// Extension of [`Erc1155`] that allows token holders to destroy both their
/// own tokens and those that they have been approved to use.
pub trait IErc1155Burnable {
    /// The error type associated to this ERC-1155 burnable trait
    /// implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// The approval is cleared when the token is burned. Relies on the `_burn`
    /// mechanism.
    ///
    /// # Arguments
    ///
    /// * `account` - Account to burn tokens from.
    /// * `token_id` - Token id to be burnt.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// If the caller is not account address and the account has not been
    /// approved, then the error [`Error::MissingApprovalForAll`] is
    /// returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    /// * The caller or account must own `token_id` or be an approved operator.
    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Self::Error>;

    /// The approval is cleared when the token is burned. Relies on the
    /// `_burn_batch` mechanism.
    ///
    /// # Arguments
    ///
    /// * `account` - Accounts to burn tokens from.
    /// * `values` - All amount to be burnt.
    /// * `token_ids` - All token id to be burnt.
    ///
    /// # Errors
    ///
    /// If the caller is not account address and the account has not been
    /// approved, then the error [`Error::MissingApprovalForAll`] is
    /// returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    /// * The caller or account must own `token_id` or be an approved operator.
    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Self::Error>;
}

impl IErc1155Burnable for Erc1155 {
    type Error = Error;

    fn burn(
        &mut self,
        account: Address,
        token_id: U256,
        value: U256,
    ) -> Result<(), Self::Error> {
        let sender = msg::sender();
        if account != sender && !self.is_approved_for_all(account, sender) {
            return Err(Error::MissingApprovalForAll(
                ERC1155MissingApprovalForAll {
                    owner: account,
                    operator: sender,
                },
            ));
        }
        self._burn(account, token_id, value)?;
        Ok(())
    }

    fn burn_batch(
        &mut self,
        account: Address,
        token_ids: Vec<U256>,
        values: Vec<U256>,
    ) -> Result<(), Self::Error> {
        let sender = msg::sender();
        if account != sender && !self.is_approved_for_all(account, sender) {
            return Err(Error::MissingApprovalForAll(
                ERC1155MissingApprovalForAll {
                    owner: account,
                    operator: sender,
                },
            ));
        }
        self._burn_batch(account, token_ids, values)?;
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use stylus_sdk::msg;

    use super::IErc1155Burnable;
    use crate::token::erc1155::{
        ERC1155InvalidSender, ERC1155MissingApprovalForAll, Erc1155, Error,
        IErc1155,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    pub(crate) fn random_token_ids(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u32>())).collect()
    }

    pub(crate) fn random_values(size: usize) -> Vec<U256> {
        (0..size).map(|_| U256::from(rand::random::<u128>())).collect()
    }

    fn init(
        contract: &mut Erc1155,
        reciever: Address,
        size: usize,
    ) -> (Vec<U256>, Vec<U256>) {
        let token_ids = random_token_ids(size);
        let values = random_values(size);

        contract
            ._mint_batch(
                reciever,
                token_ids.clone(),
                values.clone(),
                vec![0, 1, 2, 3].into(),
            )
            .expect("Mint failed");
        (token_ids, values)
    }

    #[motsu::test]
    fn test_burns(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 1);

        let initial_balance = contract
            .balance_of(BOB, token_ids[0])
            .expect("should return the BOB's balance of token 0");

        assert_eq!(values[0], initial_balance);

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        contract
            .burn(BOB, token_ids[0], values[0])
            .expect("should burn alice's token");

        let balance = contract
            .balance_of(BOB, token_ids[0])
            .expect("should return the BOB's balance of token 0");

        assert_eq!(U256::ZERO, balance);
    }

    #[motsu::test]
    fn test_error_missing_approval_when_burn(contract: Erc1155) {
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
    fn test_error_invalid_sender_when_burn(contract: Erc1155) {
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
            .expect_err("should not burn tokens from the zero address");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender,
            }) if sender == invalid_sender
        ));
    }

    #[motsu::test]
    fn test_burns_batch(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 4);

        let initial_balance_0 = contract
            .balance_of(BOB, token_ids[0])
            .expect("should return the BOB's balance of token 0");
        let initial_balance_1 = contract
            .balance_of(BOB, token_ids[1])
            .expect("should return the BOB's balance of token 1");
        let initial_balance_2 = contract
            .balance_of(BOB, token_ids[2])
            .expect("should return the BOB's balance of token 2");
        let initial_balance_3 = contract
            .balance_of(BOB, token_ids[3])
            .expect("should return the BOB's balance of token 3");

        assert_eq!(values[0], initial_balance_0);
        assert_eq!(values[1], initial_balance_1);
        assert_eq!(values[2], initial_balance_2);
        assert_eq!(values[3], initial_balance_3);

        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        contract
            .burn_batch(BOB, token_ids.clone(), values.clone())
            .expect("should burn alice's tokens");

        let balance_0 = contract
            .balance_of(BOB, token_ids[0])
            .expect("should return the BOB's balance of token 0");
        let balance_1 = contract
            .balance_of(BOB, token_ids[1])
            .expect("should return the BOB's balance of token 1");
        let balance_2 = contract
            .balance_of(BOB, token_ids[2])
            .expect("should return the BOB's balance of token 2");
        let balance_3 = contract
            .balance_of(BOB, token_ids[3])
            .expect("should return the BOB's balance of token 3");

        assert_eq!(U256::ZERO, balance_0);
        assert_eq!(U256::ZERO, balance_1);
        assert_eq!(U256::ZERO, balance_2);
        assert_eq!(U256::ZERO, balance_3);
    }

    #[motsu::test]
    fn test_error_missing_approval_when_burn_batch(contract: Erc1155) {
        let alice = msg::sender();
        let (token_ids, values) = init(contract, BOB, 2);

        let err = contract
            .burn_batch(BOB, token_ids.clone(), values.clone())
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
    fn test_error_invalid_sender_when_burn_batch(contract: Erc1155) {
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
            .expect_err("should not burn tokens from the zero address");

        assert!(matches!(
            err,
            Error::InvalidSender(ERC1155InvalidSender {
                sender,
            }) if sender == invalid_sender
        ));
    }
}
