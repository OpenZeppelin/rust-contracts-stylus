//! Optional Burnable extension of the ERC-721 standard.

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::msg;

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
    fn burn(
        &mut self,
        token_id: U256,
    ) -> Result<(), <Self as IErc721Burnable>::Error>;
}

impl IErc721Burnable for Erc721 {
    type Error = erc721::Error;

    fn burn(
        &mut self,
        token_id: U256,
    ) -> Result<(), <Self as IErc721Burnable>::Error> {
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
    use alloy_primitives::{uint, Address};
    use motsu::prelude::*;
    use stylus_sdk::{abi::Bytes, prelude::*};

    use super::*;
    use crate::token::erc721::{
        self, ERC721InsufficientApproval, ERC721NonexistentToken, Erc721,
        Error, IErc721,
    };

    const TOKEN_ID: U256 = uint!(1_U256);

    #[entrypoint]
    #[storage]
    struct Erc721Example {
        #[borrow]
        erc721: Erc721,
    }

    #[public]
    #[implements(IErc721<Error=erc721::Error>, IErc721Burnable<Error=erc721::Error>)]
    impl Erc721Example {
        fn mint(
            &mut self,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721._mint(to, token_id)
        }

        fn safe_mint(
            &mut self,
            to: Address,
            token_id: U256,
            data: Bytes,
        ) -> Result<(), erc721::Error> {
            self.erc721._safe_mint(to, token_id, &data)
        }
    }

    #[public]
    impl IErc721 for Erc721Example {
        type Error = erc721::Error;

        fn balance_of(&self, owner: Address) -> Result<U256, erc721::Error> {
            self.erc721.balance_of(owner)
        }

        fn owner_of(&self, token_id: U256) -> Result<Address, erc721::Error> {
            self.erc721.owner_of(token_id)
        }

        fn safe_transfer_from(
            &mut self,
            from: Address,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721.safe_transfer_from(from, to, token_id)
        }

        fn safe_transfer_from_with_data(
            &mut self,
            from: Address,
            to: Address,
            token_id: U256,
            data: Bytes,
        ) -> Result<(), erc721::Error> {
            self.erc721.safe_transfer_from_with_data(from, to, token_id, data)
        }

        fn transfer_from(
            &mut self,
            from: Address,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721.transfer_from(from, to, token_id)
        }

        fn approve(
            &mut self,
            to: Address,
            token_id: U256,
        ) -> Result<(), erc721::Error> {
            self.erc721.approve(to, token_id)
        }

        fn set_approval_for_all(
            &mut self,
            operator: Address,
            approved: bool,
        ) -> Result<(), erc721::Error> {
            self.erc721.set_approval_for_all(operator, approved)
        }

        fn get_approved(
            &self,
            token_id: U256,
        ) -> Result<Address, erc721::Error> {
            self.erc721.get_approved(token_id)
        }

        fn is_approved_for_all(
            &self,
            owner: Address,
            operator: Address,
        ) -> bool {
            self.erc721.is_approved_for_all(owner, operator)
        }
    }

    #[public]
    impl IErc721Burnable for Erc721Example {
        type Error = erc721::Error;

        fn burn(&mut self, token_id: U256) -> Result<(), erc721::Error> {
            self.erc721.burn(token_id)
        }
    }

    #[motsu::test]
    fn burns(contract: Contract<Erc721Example>, alice: Address) {
        let one = uint!(1_U256);

        contract
            .sender(alice)
            .mint(alice, TOKEN_ID)
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
        contract: Contract<Erc721Example>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .mint(bob, TOKEN_ID)
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
        contract: Contract<Erc721Example>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .mint(bob, TOKEN_ID)
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
        contract: Contract<Erc721Example>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .mint(alice, TOKEN_ID)
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
        contract: Contract<Erc721Example>,
        alice: Address,
        bob: Address,
    ) {
        contract
            .sender(alice)
            .mint(bob, TOKEN_ID)
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
    fn error_when_burn_nonexistent_token(
        contract: Contract<Erc721Example>,
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
