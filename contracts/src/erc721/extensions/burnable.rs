//! Optional Burnable extension of the [`ERC-721`] standard.

use alloy_primitives::{Address, U256};
use stylus_sdk::msg;

use crate::erc721::{Erc721, Error};

/// An [`Erc721`] token that can be burned (destroyed).
pub trait IErc721Burnable {
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
    /// * If token does not exist then [`Error::NonexistentToken`] is returned.
    /// * If the caller does not have the right to approve then
    ///   [`Error::InsufficientApproval`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    /// * The caller must own tokenId or be an approved operator.
    ///
    /// # Events
    ///
    /// Emits a [`Transfer`] event.
    fn burn(&mut self, token_id: U256) -> Result<(), Error>;
}

impl IErc721Burnable for Erc721 {
    fn burn(&mut self, token_id: U256) -> Result<(), Error> {
        // Setting an "auth" arguments enables the [`Erc721::_is_authorized`]
        // check which verifies that the token exists (from != `Address::ZERO`).
        //
        // Therefore, it is not needed to verify
        // that the return value is not 0 here.
        self._update(Address::ZERO, token_id, msg::sender())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{address, Address, U256};
    use once_cell::sync::Lazy;
    use stylus_sdk::msg;

    use super::IErc721Burnable;
    use crate::erc721::{tests::random_token_id, Erc721, Error, IErc721};

    // NOTE: Alice is always the sender of the message
    static ALICE: Lazy<Address> = Lazy::new(msg::sender);

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    #[grip::test]
    fn burns(contract: Erc721) {
        let one = U256::from(1);
        let token_id = random_token_id();

        contract._mint(*ALICE, token_id).expect("Mint a token for Alice");

        let initial_balance =
            contract.balance_of(*ALICE).expect("Get the balance of Alice");

        let result = contract.burn(token_id);
        let balance =
            contract.balance_of(*ALICE).expect("Get the balance of Alice");

        let err = contract.owner_of(token_id).unwrap_err();

        assert!(matches!(err, Error::NonexistentToken(_)));

        assert!(result.is_ok());

        assert_eq!(initial_balance - one, balance);
    }

    #[grip::test]
    fn get_approved_errors_when_previous_approval_burned(contract: Erc721) {
        let token_id = random_token_id();

        contract._mint(*ALICE, token_id).expect("Mint a token for Alice");
        contract.approve(BOB, token_id).expect("Approve a token for Bob");

        contract.burn(token_id).expect("Burn previously minted token");

        let err = contract.get_approved(token_id).unwrap_err();

        assert!(matches!(err, Error::NonexistentToken(_)));
    }

    #[grip::test]
    fn burn_errors_when_no_previous_approval(contract: Erc721) {
        let token_id = random_token_id();

        contract._mint(BOB, token_id).expect("Mint a token for Bob");

        let err = contract.burn(token_id).unwrap_err();

        assert!(matches!(err, Error::InsufficientApproval(_)));
    }

    #[grip::test]
    fn burn_errors_when_unknown_token(contract: Erc721) {
        let token_id = random_token_id();

        contract._mint(*ALICE, token_id).expect("Mint a token for Alice");

        let err = contract.burn(token_id + U256::from(1)).unwrap_err();

        assert!(matches!(err, Error::NonexistentToken(_)));
    }
}
