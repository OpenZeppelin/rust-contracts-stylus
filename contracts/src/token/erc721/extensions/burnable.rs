//! Optional Burnable extension of the ERC-721 standard.
use alloc::vec::Vec;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{msg, prelude::*};

use crate::token::erc721::{self, Erc721};

/// An [`Erc721`] token that can be burned (destroyed).
#[interface_id]
pub trait IErc721Burnable {
    /// The error type associated to this trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Burns `token_id`.
    /// The approval is cleared when the token is burned.
    ///
    /// # Arguments
    ///
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`erc721::Error::NonexistentToken`] - If token does not exist.
    /// * [`erc721::Error::InsufficientApproval`] - If the caller does not have
    ///   the right to approve.
    ///
    /// # Events
    ///
    /// * [`erc721::Transfer`].
    fn burn(&mut self, token_id: U256) -> Result<(), Self::Error>;
}

#[public]
impl IErc721Burnable for Erc721 {
    type Error = erc721::Error;

    fn burn(&mut self, token_id: U256) -> Result<(), Self::Error> {
        // Setting an "auth" arguments enables the
        // [`super::super::Erc721::_is_authorized`] check which verifies that
        // the token exists (from != [`Address::ZERO`]).
        //
        // Therefore, it is not needed to verify that the return value is not 0
        // here.
        self._update(Address::ZERO, token_id, msg::sender())?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use alloy_primitives::{uint, Address};
    use motsu::prelude::*;

    use super::*;
    use crate::token::erc721::{
        ERC721InsufficientApproval, ERC721NonexistentToken, Erc721, Error,
        IErc721,
    };

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn burn_succeeds_destroying_owned_token(
        contract: Contract<Erc721>,
        alice: Address,
    ) {
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
    fn burn_succeeds_destroying_approved_token(
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
    fn burn_succeeds_destroying_operator_approved_token(
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
    fn get_approved_reverts_when_token_burned(
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
    fn burn_reverts_when_approval_missing(
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
            }) if operator == alice && t_id == TOKEN_ID
        ));
    }

    #[motsu::test]
    fn burn_reverts_when_token_nonexistent(
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
