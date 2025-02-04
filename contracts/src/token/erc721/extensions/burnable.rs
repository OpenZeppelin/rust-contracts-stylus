//! Optional Burnable extension of the ERC-721 standard.

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::token::erc721::{self, Erc721};

/// An [`Erc721`] token that can be burned (destroyed).
pub trait IErc721Burnable {
    /// The error type associated to this ERC-721 burnable trait implementation.
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

impl IErc721Burnable for Erc721 {
    type Error = erc721::Error;

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
    use alloy_primitives::{address, uint, Address, U256};
    use stylus_sdk::msg;

    use super::IErc721Burnable;
    use crate::token::erc721::{
        ERC721InsufficientApproval, ERC721NonexistentToken, Erc721, Error,
        IErc721,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    const TOKEN_ID: U256 = uint!(1_U256);

    #[motsu::test]
    fn burns(contract: Erc721) {
        let alice = msg::sender();
        let one = uint!(1_U256);

        contract._mint(alice, TOKEN_ID).expect("should mint a token for Alice");

        let initial_balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let result = contract.burn(TOKEN_ID);
        assert!(result.is_ok());

        let balance = contract
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(initial_balance - one, balance);

        let err = contract
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
    fn burns_with_approval(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(BOB, TOKEN_ID).expect("should mint a token for Bob");

        let initial_balance =
            contract.balance_of(BOB).expect("should return the balance of Bob");

        contract._token_approvals.setter(TOKEN_ID).set(alice);

        let result = contract.burn(TOKEN_ID);
        assert!(result.is_ok());

        let err = contract
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));

        let balance =
            contract.balance_of(BOB).expect("should return the balance of Bob");

        assert_eq!(initial_balance - uint!(1_U256), balance);
    }

    #[motsu::test]
    fn burns_with_approval_for_all(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(BOB, TOKEN_ID).expect("should mint a token for Bob");

        let initial_balance =
            contract.balance_of(BOB).expect("should return the balance of Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract._operator_approvals.setter(BOB).setter(alice).set(true);

        let result = contract.burn(TOKEN_ID);

        assert!(result.is_ok());

        let err = contract
            .owner_of(TOKEN_ID)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == TOKEN_ID
        ));

        let balance =
            contract.balance_of(BOB).expect("should return the balance of Bob");

        assert_eq!(initial_balance - uint!(1_U256), balance);
    }

    #[motsu::test]
    fn error_when_get_approved_of_previous_approval_burned(contract: Erc721) {
        let alice = msg::sender();

        contract._mint(alice, TOKEN_ID).expect("should mint a token for Alice");
        contract
            .approve(BOB, TOKEN_ID)
            .expect("should approve a token for Bob");

        contract.burn(TOKEN_ID).expect("should burn previously minted token");

        let err = contract
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
    fn error_when_burn_without_approval(contract: Erc721) {
        contract._mint(BOB, TOKEN_ID).expect("should mint a token for Bob");

        let err = contract
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
    fn error_when_burn_nonexistent_token(contract: Erc721) {
        let err = contract
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
