//! Optional Burnable extension of the ERC-721 standard.
use core::marker::PhantomData;

use alloy_primitives::{Address, U256};
use openzeppelin_stylus_proc::r#override;
use stylus_proc::{external, sol_storage};
use stylus_sdk::{msg, prelude::*};

use crate::token::erc721::{base::IErc721Virtual, Error};

/// An [`Erc721`] token that can be burned (destroyed).
sol_storage! {
    pub struct Erc721Burnable<V: IErc721Virtual> {
        PhantomData<V> _phantom_data;
    }
}

#[external]
impl<V: IErc721Virtual> Erc721Burnable<V> {
    /// Burns `token_id`.
    /// The approval is cleared when the token is burned.
    /// Relies on the `_burn` mechanism.
    ///
    /// # Arguments
    ///
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// If token does not exist, then the error
    /// [`Error::NonexistentToken`] is returned.
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
    /// Emits a [`Transfer`] event.
    fn burn(
        storage: &mut impl TopLevelStorage,
        token_id: U256,
    ) -> Result<(), Error> {
        V::update::<V>(storage, Address::ZERO, token_id, msg::sender())?;
        Ok(())
    }
}

#[r#override]
impl IErc721Virtual for Erc721BurnableOverride {}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address};
    use stylus_sdk::msg;

    use crate::token::erc721::{
        extensions::burnable::Erc721Burnable,
        tests::{random_token_id, Override, Token},
        traits::IErc721,
        ERC721InsufficientApproval, ERC721NonexistentToken, Erc721, Error,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    #[motsu::test]
    fn burns(contract: Token) {
        let alice = msg::sender();
        let one = uint!(1_U256);
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for Alice");

        let initial_balance = contract
            .erc721
            .balance_of(alice)
            .expect("should return the balance of Alice");

        let result = Erc721Burnable::<Override>::burn(contract, token_id);
        assert!(result.is_ok());

        let balance = contract
            .erc721
            .balance_of(alice)
            .expect("should return the balance of Alice");

        assert_eq!(initial_balance - one, balance);

        let err = contract
            .erc721
            .owner_of(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn burns_with_approval(contract: Token) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token for Bob");

        let initial_balance = contract
            .erc721
            .balance_of(BOB)
            .expect("should return the balance of Bob");

        contract.erc721._token_approvals.setter(token_id).set(alice);

        let result = Erc721Burnable::<Override>::burn(contract, token_id);
        assert!(result.is_ok());

        let err = contract
            .erc721
            .owner_of(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));

        let balance = contract
            .erc721
            .balance_of(BOB)
            .expect("should return the balance of Bob");

        assert_eq!(initial_balance - uint!(1_U256), balance);
    }

    #[motsu::test]
    fn burns_with_approval_for_all(contract: Token) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token for Bob");

        let initial_balance = contract
            .erc721
            .balance_of(BOB)
            .expect("should return the balance of Bob");

        // As we cannot change `msg::sender()`, we need to use this workaround.
        contract.erc721._operator_approvals.setter(BOB).setter(alice).set(true);

        let result = Erc721Burnable::<Override>::burn(contract, token_id);

        assert!(result.is_ok());

        let err = contract
            .erc721
            .owner_of(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));

        let balance = contract
            .erc721
            .balance_of(BOB)
            .expect("should return the balance of Bob");

        assert_eq!(initial_balance - uint!(1_U256), balance);
    }

    #[motsu::test]
    fn error_when_get_approved_of_previous_approval_burned(contract: Token) {
        let alice = msg::sender();
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, alice, token_id)
            .expect("should mint a token for Alice");
        Erc721::<Override>::approve(contract, BOB, token_id)
            .expect("should approve a token for Bob");

        Erc721Burnable::<Override>::burn(contract, token_id)
            .expect("should burn previously minted token");

        let err = contract
            .erc721
            .get_approved(token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_burn_without_approval(contract: Token) {
        let token_id = random_token_id();

        Erc721::<Override>::_mint(contract, BOB, token_id)
            .expect("should mint a token for Bob");

        let err = Erc721Burnable::<Override>::burn(contract, token_id)
            .expect_err("should not burn unapproved token");

        assert!(matches!(
            err,
            Error::InsufficientApproval(ERC721InsufficientApproval {
                    operator,
                    token_id: t_id,
            }) if operator == msg::sender() && t_id == token_id
        ));
    }

    #[motsu::test]
    fn error_when_burn_nonexistent_token(contract: Token) {
        let token_id = random_token_id();

        let err = Erc721Burnable::<Override>::burn(contract, token_id)
            .expect_err("should return Error::NonexistentToken");

        assert!(matches!(
            err,
            Error::NonexistentToken (ERC721NonexistentToken{
                token_id: t_id
            }) if t_id == token_id
        ));
    }
}
