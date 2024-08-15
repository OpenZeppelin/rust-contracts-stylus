//!  ERC-721 token with pausable token transfers, minting and burning.
//!  
//!  Useful for scenarios such as preventing trades until the end of an
//! evaluation period, or having an emergency switch for freezing all token
//! transfers in the event of a large bug.
//!  
//!  IMPORTANT: This contract does not include public pause, unpause and paused
//! functions. In addition to inheriting this contract, you must define both
//! functions, invoking the [`Pausable::pause`], [`Pausable::unpause`] and
//! [`Pausable::paused`] internal functions.

use alloc::vec;

use alloy_primitives::{Address, U256};
use stylus_proc::{external, sol_storage, SolidityError};
use stylus_sdk::{abi::Bytes, msg, prelude::TopLevelStorage};

use crate::{
    token::{
        erc721,
        erc721::{
            ERC721IncorrectOwner, ERC721InvalidReceiver, ERC721InvalidSender,
            ERC721NonexistentToken, Erc721, IErc721,
        },
    },
    utils::Pausable,
};

sol_storage! {
    /// State of an [`Erc721Pausable`] token.
    pub struct Erc721Pausable {
        /// Erc721 contract storage.
        Erc721 erc721;
        /// Pausable contract storage.
        Pausable pausable;
    }
}

/// An [`Erc721Pausable`] error.
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Error type from [`Erc721`] contract [`erc721::Error`].
    Erc721(erc721::Error),
    /// Error type from pausable contract [`crate::utils::pausable::Error`].
    Pausable(crate::utils::pausable::Error),
}

unsafe impl TopLevelStorage for Erc721Pausable {}

// ************** ERC-721 External **************

#[external]
impl IErc721 for Erc721Pausable {
    type Error = Error;

    fn balance_of(&self, owner: Address) -> Result<U256, Error> {
        Ok(self.erc721.balance_of(owner)?)
    }

    fn owner_of(&self, token_id: U256) -> Result<Address, Error> {
        Ok(self.erc721.owner_of(token_id)?)
    }

    fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        // TODO: Once the SDK supports the conversion,
        // use alloy_primitives::bytes!("") here.
        self.safe_transfer_from_with_data(from, to, token_id, vec![].into())
    }

    #[selector(name = "safeTransferFrom")]
    fn safe_transfer_from_with_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self.transfer_from(from, to, token_id)?;
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            from,
            to,
            token_id,
            &data,
        )?)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO },
            )
            .into());
        }

        // Setting an "auth" argument enables the `_is_authorized` check which
        // verifies that the token exists (`!from.is_zero()`). Therefore, it is
        // not needed to verify that the return value is not 0 here.
        let previous_owner = self._update(to, token_id, msg::sender())?;
        if previous_owner != from {
            return Err(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            })
            .into());
        }
        Ok(())
    }

    fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        Ok(self.erc721._approve(to, token_id, msg::sender(), true)?)
    }

    fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Error> {
        Ok(self.erc721.set_approval_for_all(operator, approved)?)
    }

    fn get_approved(&self, token_id: U256) -> Result<Address, Error> {
        self.erc721._require_owned(token_id)?;
        Ok(self.erc721._get_approved(token_id))
    }

    fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.erc721.is_approved_for_all(owner, operator)
    }
}

// ************** Pausable **************

impl Erc721Pausable {
    /// Override of [`Erc721::_update`] that restricts normal minting to after
    /// construction.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `auth` - Account used for authorization of the update.
    ///
    /// # Errors
    ///
    /// If token does not exist and `auth` is not `Address::ZERO`, then the
    /// error [`erc721::Error::NonexistentToken`] is returned.
    /// If `auth` is not `Address::ZERO` and `auth` does not have a right to
    /// approve this token, then the error
    /// [`erc721::Error::InsufficientApproval`] is returned.
    /// TODO#q: pausable error
    ///
    /// # Events
    ///
    /// Emits a [`erc721::Transfer`] event.
    pub fn _update(
        &mut self,
        to: Address,
        token_id: U256,
        auth: Address,
    ) -> Result<Address, Error> {
        self.pausable.when_not_paused()?;

        Ok(self.erc721._update(to, token_id, auth)?)
    }
}

// ************** ERC-721 Internal **************

impl Erc721Pausable {
    /// Mints `token_id` and transfers it to `to`.
    ///
    /// WARNING: Usage of this method is discouraged, use [`Self::_safe_mint`]
    /// whenever possible.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If `token_id` already exists, then the error
    /// [`erc721::Error::InvalidSender`] is returned.
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * `to` cannot be `Address::ZERO`.
    ///
    /// # Events
    ///
    /// Emits a [`erc721::Transfer`] event.
    pub fn _mint(&mut self, to: Address, token_id: U256) -> Result<(), Error> {
        if to.is_zero() {
            return Err(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO },
            )
            .into());
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if !previous_owner.is_zero() {
            return Err(erc721::Error::InvalidSender(ERC721InvalidSender {
                sender: Address::ZERO,
            })
            .into());
        }
        Ok(())
    }

    /// Mints `token_id`, transfers it to `to`,
    /// and checks for `to`'s acceptance.
    ///
    /// An additional `data` parameter is forwarded to
    /// [`erc721::IERC721Receiver::on_erc_721_received`] to contract recipients.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Erc721::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// If `token_id` already exists, then the error
    /// [`erc721::Error::InvalidSender`] is returned.
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If [`erc721::IERC721Receiver::on_erc_721_received`] hasn't returned its
    /// interface id or returned with error, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must not exist.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`erc721::IERC721Receiver::on_erc_721_received`], which is called upon
    ///   a `safe_transfer`.
    ///
    /// # Events
    ///
    /// Emits a [`erc721::Transfer`] event.
    pub fn _safe_mint(
        &mut self,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self._mint(to, token_id)?;
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            Address::ZERO,
            to,
            token_id,
            &data,
        )?)
    }

    /// Destroys `token_id`.
    ///
    /// The approval is cleared when the token is burned. This is an
    /// internal function that does not check if the sender is authorized
    /// to operate on the token.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If token does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `token_id` must exist.
    ///
    /// # Events
    ///
    /// Emits a [`erc721::Transfer`] event.
    pub fn _burn(&mut self, token_id: U256) -> Result<(), Error> {
        let previous_owner =
            self._update(Address::ZERO, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(erc721::Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
        }
        Ok(())
    }

    /// Transfers `token_id` from `from` to `to`.
    ///
    /// As opposed to [`Self::transfer_from`], this imposes no restrictions on
    /// `msg::sender`.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If `token_id` does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    /// If the previous owner is not `from`, then  the error
    /// [`erc721::Error::IncorrectOwner`] is returned.
    ///
    /// # Requirements:
    ///
    /// * `to` cannot be `Address::ZERO`.
    /// * The `token_id` token must be owned by `from`.
    ///
    /// # Events
    ///
    /// Emits a [`erc721::Transfer`] event.
    pub fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(erc721::Error::InvalidReceiver(
                ERC721InvalidReceiver { receiver: Address::ZERO },
            )
            .into());
        }

        let previous_owner = self._update(to, token_id, Address::ZERO)?;
        if previous_owner.is_zero() {
            return Err(erc721::Error::NonexistentToken(
                ERC721NonexistentToken { token_id },
            )
            .into());
        } else if previous_owner != from {
            return Err(erc721::Error::IncorrectOwner(ERC721IncorrectOwner {
                sender: from,
                token_id,
                owner: previous_owner,
            })
            .into());
        }

        Ok(())
    }

    /// Safely transfers `token_id` token from `from` to `to`, checking that
    /// contract recipients are aware of the [`Erc721`] standard to prevent
    /// tokens from being forever locked.
    ///
    /// `data` is additional data, it has
    /// no specified format and it is sent in call to `to`. This internal
    /// function is like [`Self::safe_transfer_from`] in the sense that it
    /// invokes [`erc721::IERC721Receiver::on_erc_721_received`] on the
    /// receiver, and can be used to e.g. implement alternative mechanisms
    /// to perform token transfer, such as signature-based.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account of the sender.
    /// * `to` - Account of the recipient.
    /// * `token_id` - Token id as a number.
    /// * `data` - Additional data with no specified format, sent in the call to
    ///   [`Erc721::_check_on_erc721_received`].
    ///
    /// # Errors
    ///
    /// If `to` is `Address::ZERO`, then the error
    /// [`erc721::Error::InvalidReceiver`] is returned.
    /// If `token_id` does not exist, then the error
    /// [`erc721::Error::NonexistentToken`] is returned.
    /// If the previous owner is not `from`, then the error
    /// [`erc721::Error::IncorrectOwner`] is returned.
    ///
    /// # Requirements:
    ///
    /// * The `token_id` token must exist and be owned by `from`.
    /// * `to` cannot be `Address::ZERO`.
    /// * `from` cannot be `Address::ZERO`.
    /// * If `to` refers to a smart contract, it must implement
    ///   [`erc721::IERC721Receiver::on_erc_721_received`], which is called upon
    ///   a `safe_transfer`.
    ///
    /// # Events
    ///
    /// Emits a [`erc721::Transfer`] event.
    pub fn _safe_transfer(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), Error> {
        self._transfer(from, to, token_id)?;
        Ok(self.erc721._check_on_erc721_received(
            msg::sender(),
            from,
            to,
            token_id,
            &data,
        )?)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use alloy_primitives::{address, uint, Address};
    use stylus_sdk::msg;

    use crate::{
        token::{
            erc721,
            erc721::{
                extensions::pausable::{Erc721Pausable, Error},
                tests::random_token_id,
                ERC721NonexistentToken, IErc721,
            },
        },
        utils::pausable,
    };

    const BOB: Address = address!("F4EaCDAbEf3c8f1EdE91b6f2A6840bc2E4DD3526");

    impl Erc721Pausable {
        fn init(&mut self, paused: bool) {
            self.pausable._paused.set(paused);
        }
    }

    #[motsu::test]
    fn error_when_burn_in_paused_state(contract: Erc721Pausable) {
        contract.init(false);
        let alice = msg::sender();

        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint token");
        let initial_balance =
            contract.balance_of(alice).expect("should have balance");

        contract.pausable.pause().expect("should pause the contract");

        let err =
            contract._burn(token_id).expect_err("should fail when paused");

        assert!(matches!(
            err,
            Error::Pausable(pausable::Error::EnforcedPause(_))
        ));

        let balance = contract.balance_of(alice).expect("should have balance");
        assert_eq!(balance, initial_balance);

        let owner = contract.owner_of(token_id).expect("should have owner");
        assert_eq!(owner, alice);
    }

    #[motsu::test]
    fn error_when_mint_in_paused_state(contract: Erc721Pausable) {
        let alice = msg::sender();
        contract.init(false);

        let token_id = random_token_id();

        contract.pausable.pause().expect("should pause the contract");

        let err = contract
            ._mint(alice, token_id)
            .expect_err("should fail when paused");

        assert!(matches!(
            err,
            Error::Pausable(pausable::Error::EnforcedPause(_))
        ));

        let err = contract.owner_of(token_id).expect_err("should have owner");

        assert!(matches!(
            err,
            Error::Erc721(erc721::Error::NonexistentToken(ERC721NonexistentToken { token_id: t_id }))
            if t_id == token_id
        ));

        let balance = contract.balance_of(alice).expect("should have balance");
        assert_eq!(balance, uint!(0_U256));
    }

    #[motsu::test]
    fn error_when_transfer_in_paused_state(contract: Erc721Pausable) {
        let alice = msg::sender();
        contract.init(false);

        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint token");

        let initial_alice_balance =
            contract.balance_of(alice).expect("should have balance");

        let initial_bob_balance =
            contract.balance_of(BOB).expect("should have balance");

        contract.pausable.pause().expect("should pause the contract");
        let err = contract
            ._transfer(alice, BOB, token_id)
            .expect_err("should fail when paused");

        assert!(matches!(
            err,
            Error::Pausable(pausable::Error::EnforcedPause(_))
        ));

        let owner = contract.owner_of(token_id).expect("should have owner");
        assert_eq!(owner, alice);

        let alice_balance =
            contract.balance_of(alice).expect("should have balance");
        assert_eq!(alice_balance, initial_alice_balance);

        let bob_balance =
            contract.balance_of(BOB).expect("should have balance");
        assert_eq!(bob_balance, initial_bob_balance);
    }

    #[motsu::test]
    fn error_when_safe_transfer_in_paused_state(contract: Erc721Pausable) {
        let alice = msg::sender();
        contract.init(false);

        let token_id = random_token_id();
        contract._mint(alice, token_id).expect("should mint token");

        let initial_alice_balance =
            contract.balance_of(alice).expect("should have balance");

        let initial_bob_balance =
            contract.balance_of(BOB).expect("should have balance");

        contract.pausable.pause().expect("should pause the contract");
        let err = contract
            ._safe_transfer(alice, BOB, token_id, vec![0, 1, 2, 3].into())
            .expect_err("should fail when paused");

        assert!(matches!(
            err,
            Error::Pausable(pausable::Error::EnforcedPause(_))
        ));

        let owner = contract.owner_of(token_id).expect("should have owner");
        assert_eq!(owner, alice);

        let alice_balance =
            contract.balance_of(alice).expect("should have balance");
        assert_eq!(alice_balance, initial_alice_balance);

        let bob_balance =
            contract.balance_of(BOB).expect("should have balance");
        assert_eq!(bob_balance, initial_bob_balance);
    }
}
