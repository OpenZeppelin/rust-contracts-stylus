//! Optional Burnable extension of the ERC-721 standard.

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::token::erc721::{Erc721, Error};

/// An [`Erc721`] token that can be burned (destroyed).
pub trait IErc721Burnable {
    /// The error type associated to this ERC-721 burnable trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Burns `token_id`.
    ///
    /// The approval is cleared when the token is burned. Relies on the `_burn`
    /// mechanism.
    ///
    /// # Arguments
    ///
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// If token does not exist, then the error [`Error::NonexistentToken`] is
    /// returned.
    /// If the caller does not have the right to approve, then the error
    /// [`Error::InsufficientApproval`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    /// * The caller must own `token_id` or be an approved operator.
    ///
    /// # Events
    ///
    /// Emits a [`super::super::Transfer`] event.
    fn burn(&mut self, token_id: U256) -> Result<(), Self::Error>;
}

impl IErc721Burnable for Erc721 {
    type Error = Error;

    fn burn(&mut self, token_id: U256) -> Result<(), Self::Error> {
        // Setting an "auth" arguments enables the
        // [`super::super::Erc721::_is_authorized`] check which verifies that
        // the token exists (from != `Address::ZERO`).
        //
        // Therefore, it is not needed to verify that the return value is not 0
        // here.
        self._update(Address::ZERO, token_id, msg::sender())?;
        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{uint, Address, U256};
    use motsu::prelude::Contract;
    use stylus_sdk::msg;

    use super::IErc721Burnable;
    use crate::token::erc721::{
        ERC721InsufficientApproval, ERC721NonexistentToken, Erc721, Error,
        IErc721,
    };

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn burns(contract: Contract<Erc721>, alice: Address) {
        let one = uint!(1_U256);

        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .expect("should mint a token for Alice");

        let initial_balance = contract
            .sender(alice)
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let result = contract.sender(alice).burn(TOKEN_ID);
        assert!(result.is_ok());

        let balance = contract
            .sender(alice)
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(initial_balance - one, balance);

        let err = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn burns_with_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .expect("should mint a token for Bob");

        let initial_balance = contract
            .sender(alice)
            .balance_of(bob)
            .expect("should return the balance of Bob");

        contract
            .sender(bob)
            .approve(alice, TOKEN_ID)
            .expect("should approve a token for Alice");

        let result = contract.sender(alice).burn(TOKEN_ID);
        assert!(result.is_ok());

        let err = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));

        let balance = contract
            .sender(alice)
            .balance_of(bob)
            .expect("should return the balance of Bob");

        assert_eq!(initial_balance - uint!(1_U256), balance);
    }

    #[motsu::test]
    fn burns_with_approval_for_all(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .expect("should mint a token for Bob");

        let initial_balance = contract
            .sender(alice)
            .balance_of(bob)
            .expect("should return the balance of Bob");

        contract
            .sender(bob)
            .set_approval_for_all(alice, true)
            .expect("should approve all Bob's tokens for Alice");

        let result = contract.sender(alice).burn(TOKEN_ID);

        assert!(result.is_ok());

        let err = contract
            .sender(alice)
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));

        let balance = contract
            .sender(alice)
            .balance_of(bob)
            .expect("should return the balance of Bob");

        assert_eq!(initial_balance - uint!(1_U256), balance);
    }

    #[motsu::test]
    fn error_when_get_approved_of_previous_approval_burned(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(alice, TOKEN_ID)
            .expect("should mint a token for Alice");
        contract
            .sender(alice)
            .approve(bob, TOKEN_ID)
            .expect("should approve a token for Bob");

        contract
            .sender(alice)
            .burn(TOKEN_ID)
            .expect("should burn previously minted token");

        let err = contract
            .sender(alice)
            .get_approved(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_burn_without_approval(
        contract: Contract<Erc721>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            ._mint(bob, TOKEN_ID)
            .expect("should mint a token for Bob");

        let err = contract
            .sender(alice)
            .burn(TOKEN_ID)
            .expect_err("should not burn unapproved token");

        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                    operator,
                    token_id: t_id,
            }) if operator == msg::sender() && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn error_when_burn_nonexistent_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            .burn(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));
    }
}
