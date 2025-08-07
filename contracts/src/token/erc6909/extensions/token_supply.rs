//! Implementation of the Token Supply extension defined in ERC6909.
//! Tracks the total supply of each token id individually.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    msg,
    prelude::*,
    storage::{StorageMap, StorageU256},
};

use crate::{
    token::erc6909::{self, Erc6909, Error, IErc6909},
    utils::{
        introspection::erc165::IErc165,
        math::storage::{AddAssignChecked, SubAssignUnchecked},
    },
};

/// State of an [`Erc6909TokenSupply`] contract.
#[storage]
pub struct Erc6909TokenSupply {
    /// [`Erc6909`] contract.
    pub erc6909: Erc6909,
    /// Mapping from token id to total supply.
    pub(crate) total_supplies: StorageMap<U256, StorageU256>,
}

/// Required interface of a [`Erc6909TokenSupply`] contract.
#[interface_id]
pub trait IErc6909TokenSupply: IErc165 {
    /// Returns the total supply of the token of type `id`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `id` - Token id as a number.
    fn total_supply(&self, id: U256) -> U256;
}

#[public]
#[implements(IErc6909<Error = Error>, IErc6909TokenSupply, IErc165)]
impl Erc6909TokenSupply {}

#[public]
impl IErc6909TokenSupply for Erc6909TokenSupply {
    fn total_supply(&self, id: U256) -> U256 {
        self.total_supplies.get(id)
    }
}

#[public]
impl IErc6909 for Erc6909TokenSupply {
    type Error = erc6909::Error;

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.erc6909.balance_of(owner, id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.erc6909.allowance(owner, spender, id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.erc6909.is_operator(owner, spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        self.erc6909.approve(spender, id, amount)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        self.erc6909.set_operator(spender, approved)
    }

    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let sender = msg::sender();
        self._transfer(sender, receiver, id, amount)?;
        Ok(true)
    }

    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let caller = msg::sender();
        if (sender != caller) && !self.is_operator(sender, caller) {
            self.erc6909._spend_allowance(sender, caller, id, amount)?;
        }
        self._transfer(sender, receiver, id, amount)?;
        Ok(true)
    }
}

impl Erc6909TokenSupply {
    /// Creates `amount` of token `id` and assigns them to `account`, by
    /// transferring it from [`Address::ZERO`]. Relies on the `_update`
    /// mechanism.
    ///
    /// Re-export of [`Erc6909::_mint`].
    #[allow(clippy::missing_errors_doc)]
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(erc6909::Error::InvalidReceiver(
                erc6909::ERC6909InvalidReceiver { receiver: Address::ZERO },
            ));
        }

        self._update(Address::ZERO, to, id, amount)
    }

    /// Destroys a `amount` of token `id` from `account`.
    /// Relies on the `_update` mechanism.
    ///
    /// Re-export of [`Erc6909::_burn`].
    #[allow(clippy::missing_errors_doc)]
    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(erc6909::Error::InvalidSender(
                erc6909::ERC6909InvalidSender { sender: Address::ZERO },
            ));
        }

        self._update(from, Address::ZERO, id, amount)
    }

    /// Moves `amount` of token `id` from `from` to `to` without checking for
    /// approvals. This function verifies that neither the sender nor the
    /// receiver are [`Address::ZERO`], which means it cannot mint or burn
    /// tokens.
    ///
    /// Relies on the `_update` mechanism.
    ///
    /// Re-export of [`Erc6909::_transfer`].
    #[allow(clippy::missing_errors_doc)]
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(erc6909::Error::InvalidSender(
                erc6909::ERC6909InvalidSender { sender: Address::ZERO },
            ));
        }

        if to.is_zero() {
            return Err(erc6909::Error::InvalidReceiver(
                erc6909::ERC6909InvalidReceiver { receiver: Address::ZERO },
            ));
        }

        self._update(from, to, id, amount)?;

        Ok(())
    }
}

impl Erc6909TokenSupply {
    /// Extended version of [`Erc6909::_update`] that updates the supply of
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InsufficientBalance`] - If the `from` address doesn't have
    ///   enough tokens.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    ///
    /// # Panics
    ///
    /// * If updated balance and/or supply exceeds [`U256::MAX`], may happen
    ///   during the `mint` operation.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), erc6909::Error> {
        self.erc6909._update(from, to, id, amount)?;

        if from.is_zero() {
            self.total_supplies.setter(id).add_assign_checked(
                amount,
                "should not exceed `U256::MAX` for `total_supplies`",
            );
        }

        if to.is_zero() {
            self.total_supplies.setter(id).sub_assign_unchecked(amount);
        }
        Ok(())
    }
}

#[public]
impl IErc165 for Erc6909TokenSupply {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc6909TokenSupply>::interface_id() == interface_id
            || self.erc6909.supports_interface(interface_id)
    }
}

#[cfg(test)]
mod tests {
    use motsu::prelude::*;
    use stylus_sdk::{
        alloy_primitives::{fixed_bytes, uint, Address, FixedBytes, U256},
        prelude::*,
    };

    use super::*;
    use crate::token::erc6909::{ERC6909InvalidSender};

    unsafe impl TopLevelStorage for Erc6909TokenSupply {}

    #[motsu::test]
    fn mint(contract: Contract<Erc6909TokenSupply>, alice: Address) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(id));

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        assert_eq!(ten, contract.sender(alice).balance_of(alice, id));

        assert_eq!(ten, contract.sender(alice).total_supply(id));
    }

    #[motsu::test]
    fn mint_twice(
        contract: Contract<Erc6909TokenSupply>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(1_U256);
        let five = uint!(5_U256);
        let ten = uint!(10_U256);

        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(id));

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        contract
            .sender(alice)
            ._mint(bob, id, five)
            .expect("should mint tokens for Bob");

        assert_eq!(ten, contract.sender(alice).balance_of(alice, id));

        assert_eq!(five, contract.sender(alice).balance_of(bob, id));

        assert_eq!(ten + five, contract.sender(alice).total_supply(id));
    }

    #[motsu::test]
    fn mint_errors_invalid_receiver(
        contract: Contract<Erc6909TokenSupply>,
        alice: Address,
    ) {
        let id = uint!(1_U256);
        let ten = uint!(10_U256);

        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(id));

        let invalid_receiver = Address::ZERO;

        let err = contract
            .sender(alice)
            ._mint(invalid_receiver, id, ten)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidReceiver(_)));
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `total_supplies`"]
    fn mint_panics_on_total_supply_overflow(
        contract: Contract<Erc6909TokenSupply>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = uint!(1_U256);
        let ten = uint!(10_U256);

        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(id));

        contract
            .sender(alice)
            ._mint(alice, id, U256::MAX - ten)
            .expect("should mint tokens for Alice");

        // This should panic
        contract
            .sender(alice)
            ._mint(bob, id, ten + one)
            .expect("should mint tokens for Bob");
    }

    #[motsu::test]
    fn burn(contract: Contract<Erc6909TokenSupply>, alice: Address) {
        let id = uint!(2_U256);
        let ten = uint!(10_U256);
        let one = uint!(1_U256);

        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(id));

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        contract
            .sender(alice)
            ._burn(alice, id, one)
            .expect("should burn tokens for Alice");

        assert_eq!(ten - one, contract.sender(alice).total_supply(id));
    }

    #[motsu::test]
    fn burn_errors_invalid_sender(
        contract: Contract<Erc6909TokenSupply>,
        alice: Address,
    ) {
        let id = uint!(2_U256);
        let ten = uint!(10_U256);

        let invalid_sender = Address::ZERO;

        assert_eq!(U256::ZERO, contract.sender(alice).total_supply(id));

        let err = contract
            .sender(alice)
            ._burn(invalid_sender, id, ten)
            .motsu_unwrap_err();
        assert!(
            matches!(err, Error::InvalidSender(ERC6909InvalidSender { sender }) if sender == invalid_sender)
        );
    }

    #[motsu::test]
    fn interface_id() {
        let actual =
            <Erc6909TokenSupply as IErc6909TokenSupply>::interface_id();
        let expected: FixedBytes<4> = fixed_bytes!("0xbd85b039");
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(
        contract: Contract<Erc6909TokenSupply>,
        alice: Address,
    ) {
        assert!(contract.sender(alice).supports_interface(
            <Erc6909TokenSupply as IErc6909TokenSupply>::interface_id()
        ));
        assert!(contract.sender(alice).supports_interface(
            <Erc6909TokenSupply as IErc165>::interface_id()
        ));
        assert!(contract.sender(alice).supports_interface(
            <Erc6909TokenSupply as IErc6909>::interface_id()
        ));

        let fake_interface_id = 0x12345678u32;
        assert!(!contract
            .sender(alice)
            .supports_interface(fake_interface_id.into()));
    }
}
