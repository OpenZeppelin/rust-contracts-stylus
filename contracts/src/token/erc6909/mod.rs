//! Implementation of the [ERC-6909] token standard.
//!
//! [ERC-6909]: https://eips.ethereum.org/EIPS/eip-6909

use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, Address, U256};
use openzeppelin_stylus_proc::interface_id;
use stylus_sdk::{
    call::MethodError,
    evm, msg,
    prelude::*,
    storage::{StorageBool, StorageMap, StorageU256},
};

use crate::utils::introspection::erc165::IErc165;

pub mod extensions;

pub use sol::*;

use crate::utils::math::storage::AddAssignChecked;

#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when the allowance of a `spender` for an `owner` is set for a token of type `id`.
        /// The new allowance is `amount`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Approval(address indexed owner, address indexed spender, uint256 indexed id, uint256 amount);

        /// Emitted when `owner` grants or revokes operator status for a `spender`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event OperatorSet(address indexed owner, address indexed spender, bool approved);

        /// Emitted when `amount` tokens of type `id` are moved from `sender` to `receiver` initiated by `caller`.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Transfer(address caller, address indexed sender, address indexed receiver, uint256 indexed id, uint256 amount);
    }

    sol! {
        /// Indicates an error related to the current `balance` of `sender`. Used
        /// in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        /// * `balance` - Current balance for the interacting account.
        /// * `needed` - Minimum amount required to perform a transfer.
        /// * `id`- Identifier number of a token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 id);

        /// Indicates a failure with the `spender`’s `allowance`. Used in
        /// transfers.
        ///
        /// * `spender` - Address that may be allowed to operate on tokens without
        /// being their owner.
        /// * `allowance` - Amount of tokens a `spender` is allowed to operate
        /// with.
        /// * `needed` - Minimum amount required to perform a transfer.
        /// * `id` - Identifier number of a token.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InsufficientAllowance(address spender, uint256 allowance, uint256 needed, uint256 id);

        /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
        /// approver Address initiating an approval operation.
        ///
        /// * `approver` - Address initiating an approval operation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InvalidApprover(address approver);

        /// Indicates a failure with the token `receiver`. Used in transfers.
        ///
        /// * `receiver` - Address to which the tokens are being transferred.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InvalidReceiver(address receiver);

        /// Indicates a failure with the token `sender`. Used in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InvalidSender(address sender);

        /// Indicates a failure with the `spender` to be approved. Used in
        /// approvals.
        ///
        /// * `spender` - Address that may be allowed to operate on tokens without
        /// being their owner.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC6909InvalidSpender(address spender);
    }
}

/// An [`Erc6909`] error defined as described in [ERC-6909].
///
/// [ERC-6909]: https://eips.ethereum.org/EIPS/eip-6909
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the current `balance` of `sender`. Used
    /// in transfers.
    InsufficientBalance(ERC6909InsufficientBalance),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(ERC6909InsufficientAllowance),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals.
    InvalidApprover(ERC6909InvalidApprover),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(ERC6909InvalidReceiver),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(ERC6909InvalidSender),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(ERC6909InvalidSpender),
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// State of an [`Erc6909`] token.
#[storage]
pub struct Erc6909 {
    pub(crate) balances: StorageMap<Address, StorageMap<U256, StorageU256>>,
    pub(crate) operator_approvals:
        StorageMap<Address, StorageMap<Address, StorageBool>>,
    pub(crate) allowances:
        StorageMap<Address, StorageMap<Address, StorageMap<U256, StorageU256>>>,
}

/// NOTE: Implementation of [`TopLevelStorage`] to be able use `&mut self` when
/// calling other contracts and not `&mut (impl TopLevelStorage +
/// BorrowMut<Self>)`. Should be fixed in the future by the Stylus team.
unsafe impl TopLevelStorage for Erc6909 {}

/// Required interface of an [`Erc6909`] compliant contract.
#[interface_id]
pub trait IErc6909: IErc165 {
    /// The error type associated to this ERC-6909 trait implementation.
    type Error: Into<alloc::vec::Vec<u8>>;

    /// Returns the amount of tokens of type `id` owned by `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `id` - Token id as a number.
    fn balance_of(&self, owner: Address, id: U256) -> U256;

    /// Returns the amount of tokens of type `id` that `spender` is allowed to
    /// spend on behalf of `owner`.
    ///
    /// NOTE: Does not include operator allowances.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `id` - Token id as a number.
    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256;

    /// Returns true if `spender` is set as an operator for `owner`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that is an operator for the owner.
    fn is_operator(&self, owner: Address, spender: Address) -> bool;

    /// Sets an approval to `spender` for `amount` of tokens of type `id` from
    /// the caller's tokens. An `amount` of [`U256::MAX`] signifies an
    /// unlimited approval.
    ///
    /// Must return true.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `spender` - Account that will spend the tokens.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Approval`].
    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error>;

    /// Grants or revokes unlimited transfer permission of any token id to
    /// `spender` for the caller's tokens.
    ///
    /// Must return true.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `spender` - Account that will spend the tokens.
    /// * `approved` - Flag that determines whether or not permission will be
    ///   granted to `operator`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`OperatorSet`].
    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error>;

    /// Transfers `amount` of token type `id` from the caller's account to
    /// `receiver`.
    ///
    /// Must return true.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `receiver`- Account to transfer tokens to.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `receiver` address is
    ///   [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the caller doesn't have a balance
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn transfer(
        &mut self,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error>;

    /// Transfers `amount` of token type `id` from `sender` to `receiver`.
    ///
    /// Must return true.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `sender` - Account to transfer tokens from.
    /// * `receiver` - Account to transfer tokens to.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `sender` address is
    ///   [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If the `receiver` address is
    ///   [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the `sender` doesn't have a
    ///   balance
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn transfer_from(
        &mut self,
        sender: Address,
        receiver: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error>;
}

#[public]
#[implements(IErc6909<Error = Error>, IErc165)]
impl Erc6909 {}

#[public]
impl IErc6909 for Erc6909 {
    type Error = Error;

    fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.balances.get(owner).get(id)
    }

    fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
        self.allowances.get(owner).get(spender).get(id)
    }

    fn is_operator(&self, owner: Address, spender: Address) -> bool {
        self.operator_approvals.get(owner).get(spender)
    }

    fn approve(
        &mut self,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<bool, Self::Error> {
        let owner = msg::sender();
        self._approve(owner, spender, id, amount, true)
    }

    fn set_operator(
        &mut self,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Self::Error> {
        let owner = msg::sender();
        self._set_operator(owner, spender, approved)?;
        Ok(true)
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
            self._spend_allowance(sender, caller, id, amount)?;
        }
        self._transfer(sender, receiver, id, amount)?;
        Ok(true)
    }
}

impl Erc6909 {
    /// Sets `amount` as the allowance of `spender` over the `owner`'s `id`
    /// tokens.
    ///
    /// This internal function is equivalent to [`Self::approve`], and can be
    /// used to e.g. set automatic allowances for certain subsystems, etc.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    /// * `emit_event` - Emit an [`Approval`] event flag.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    /// * [`Error::InvalidApprover`] - If the `owner` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Approval`].
    pub fn _approve(
        &mut self,
        owner: Address,
        spender: Address,
        id: U256,
        amount: U256,
        emit_event: bool,
    ) -> Result<bool, Error> {
        if owner.is_zero() {
            return Err(Error::InvalidApprover(ERC6909InvalidApprover {
                approver: Address::ZERO,
            }));
        }

        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC6909InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self.allowances.setter(owner).setter(spender).insert(id, amount);

        if emit_event {
            evm::log(Approval { owner, spender, id, amount });
        }

        Ok(true)
    }

    /// Approve `spender` to operate on all of `owner`'s tokens
    ///
    /// This internal function is equivalent to [`Self::set_operator`], and can
    /// be used to e.g. set automatic allowances for certain subsystems,
    /// etc.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `approved` - Flag that determines whether permission will be granted
    ///   to `operator`.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    /// * [`Error::InvalidApprover`] - If the `owner` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`OperatorSet`].
    pub fn _set_operator(
        &mut self,
        owner: Address,
        spender: Address,
        approved: bool,
    ) -> Result<bool, Error> {
        if owner.is_zero() {
            return Err(Error::InvalidApprover(ERC6909InvalidApprover {
                approver: Address::ZERO,
            }));
        }

        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC6909InvalidSpender {
                spender: Address::ZERO,
            }));
        }

        self.operator_approvals.setter(owner).insert(spender, approved);

        evm::log(OperatorSet { owner, spender, approved });

        Ok(true)
    }

    /// Moves `amount` of token `id` from `from` to `to` without checking for
    /// approvals. This function verifies that neither the sender nor the
    /// receiver are [`Address::ZERO`], which means it cannot mint or burn
    /// tokens.
    ///
    /// Relies on the [`Self::_update`] function.
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
    /// * [`Error::InvalidSender`] - If the `from` address is [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If the `to` address is [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the `from` address doesn't have
    ///   enough tokens.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    ///
    /// # Panics
    ///
    /// * If updated balance of `to` exceeds [`U256::MAX`].
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC6909InvalidSender {
                sender: Address::ZERO,
            }));
        }

        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC6909InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        self._update(from, to, id, amount)?;

        Ok(())
    }

    /// Transfers `amount` of token `id` from `from` to `to`, or alternatively
    /// mints (or burns) if `from` (or `to`) is the zero address.
    /// All customizations to transfers, mints, and burns should be done by
    /// overriding this function.
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
    /// * If updated balance of `to` exceeds [`U256::MAX`].
    pub fn _update(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        let caller = msg::sender();

        if !from.is_zero() {
            let from_balance = self.balances.get(from).get(id);
            if from_balance < amount {
                return Err(Error::InsufficientBalance(
                    ERC6909InsufficientBalance {
                        sender: from,
                        balance: from_balance,
                        needed: amount,
                        id,
                    },
                ));
            }
            self.balances.setter(from).setter(id).set(from_balance - amount);
        }

        if !to.is_zero() {
            self.balances.setter(to).setter(id).add_assign_checked(
                amount,
                "should not exceed `U256::MAX` for `balances`",
            );
        }

        evm::log(Transfer { caller, sender: from, receiver: to, id, amount });
        Ok(())
    }

    /// Updates `owner`'s allowance for `spender` based on spent `amount`.
    ///
    /// Does not update the allowance value in case of infinite allowance.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InsufficientAllowance`] - If the `spender` does not have
    ///   enough allowance to spend `amount` of tokens.
    /// * [`Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    /// * [`Error::InvalidApprover`] - If the `owner` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * Does not emit an [`Approval`] event.
    pub fn _spend_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        let current_allowance = self.allowance(owner, spender, id);
        if current_allowance < U256::MAX {
            if current_allowance < amount {
                return Err(Error::InsufficientAllowance(
                    ERC6909InsufficientAllowance {
                        spender,
                        allowance: current_allowance,
                        needed: amount,
                        id,
                    },
                ));
            }

            self.allowances
                .setter(owner)
                .setter(spender)
                .setter(id)
                .set(current_allowance - amount);
        }
        Ok(())
    }

    /// Creates `amount` of token `id` and assigns them to `account`, by
    /// transferring it from [`Address::ZERO`].
    /// Relies on the [`Self::_update`] function.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account to transfer tokens to.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `to` address is [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Transfer`] with `from` set to the zero address.
    ///
    /// # Panics
    ///
    /// * If updated balance of `to` exceeds [`U256::MAX`].
    pub fn _mint(
        &mut self,
        to: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC6909InvalidReceiver {
                receiver: Address::ZERO,
            }));
        }

        self._update(Address::ZERO, to, id, amount)
    }

    /// Destroys a `amount` of token `id` from `account`.
    /// Relies on the [`Self::_update`] function.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `id` - Token id as a number.
    /// * `amount` - Amount to be transferred.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `from` address is [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the `from` address doesn't have
    ///   enough tokens.
    ///
    /// # Events
    ///
    /// * [`Transfer`] with `to` set to the zero address.
    pub fn _burn(
        &mut self,
        from: Address,
        id: U256,
        amount: U256,
    ) -> Result<(), Error> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC6909InvalidSender {
                sender: Address::ZERO,
            }));
        }

        self._update(from, Address::ZERO, id, amount)
    }
}

#[public]
impl IErc165 for Erc6909 {
    fn supports_interface(&self, interface_id: B32) -> bool {
        <Self as IErc6909>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{aliases::B32, uint, Address, U256};
    use motsu::prelude::*;

    use super::{
        Approval, Erc6909, Error, IErc165, IErc6909, OperatorSet, Transfer,
    };

    #[motsu::test]
    fn mint(contract: Contract<Erc6909>, alice: Address) {
        let id = U256::ONE;
        let ten = uint!(10_U256);

        // Store the initial balance & supply.
        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        let balance = contract.sender(alice).balance_of(alice, id);

        assert_eq!(balance, ten, "Alice's balance should be 10");

        contract.assert_emitted(&Transfer {
            caller: alice,
            sender: Address::ZERO,
            receiver: alice,
            id,
            amount: ten,
        });
    }

    #[motsu::test]
    fn mint_errors_invalid_receiver(
        contract: Contract<Erc6909>,
        alice: Address,
    ) {
        let receiver = Address::ZERO;
        let id = U256::ONE;
        let ten = uint!(10_U256);

        let initial_balance = contract.sender(alice).balance_of(receiver, id);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        let err =
            contract.sender(alice)._mint(receiver, id, ten).motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidReceiver(_)));

        assert_eq!(
            initial_balance,
            contract.sender(alice).balance_of(receiver, id)
        );
    }

    #[motsu::test]
    fn burn(contract: Contract<Erc6909>, alice: Address) {
        let id = uint!(2_U256);
        let one = U256::ONE;
        let ten = uint!(10_U256);

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .expect("should mint tokens for Alice");

        let balance = contract.sender(alice).balance_of(alice, id);

        contract.sender(alice)._burn(alice, id, one).motsu_unwrap();

        assert_eq!(balance - one, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn burn_errors_insufficient_balance(
        contract: Contract<Erc6909>,
        alice: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;
        let ten = uint!(10_U256);

        contract
            .sender(alice)
            ._mint(alice, id, one)
            .expect("should mint tokens for Alice");

        let balance = contract.sender(alice).balance_of(alice, id);

        let err =
            contract.sender(alice)._burn(alice, id, ten).motsu_unwrap_err();

        assert!(matches!(err, Error::InsufficientBalance(_)));

        assert_eq!(balance, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn burn_errors_invalid_sender(contract: Contract<Erc6909>, alice: Address) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        let invalid_sender = Address::ZERO;

        let err = contract
            .sender(alice)
            ._burn(invalid_sender, id, one)
            .expect_err("should not burn token for invalid sender");

        assert!(matches!(err, Error::InvalidSender(_)));
    }

    #[motsu::test]
    fn transfer(contract: Contract<Erc6909>, alice: Address, bob: Address) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        contract
            .sender(alice)
            ._mint(alice, id, one)
            .motsu_expect("should mint tokens for Alice");

        contract
            .sender(alice)
            ._mint(bob, id, one)
            .motsu_expect("should mint tokens for Bob");

        let alice_balance = contract.sender(alice).balance_of(alice, id);
        let bob_balance = contract.sender(alice).balance_of(bob, id);

        let result = contract.sender(alice).transfer(bob, id, one);
        assert!(result.is_ok());

        assert_eq!(
            alice_balance - one,
            contract.sender(alice).balance_of(alice, id)
        );

        assert_eq!(
            bob_balance + one,
            contract.sender(alice).balance_of(bob, id)
        );

        contract.assert_emitted(&Transfer {
            caller: alice,
            sender: alice,
            receiver: bob,
            id,
            amount: one,
        });
    }

    #[motsu::test]
    fn transfer_errors_insufficient_balance(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        contract
            .sender(alice)
            ._mint(alice, id, one)
            .motsu_expect("should mint tokens for Alice");

        contract
            .sender(alice)
            ._mint(bob, id, one)
            .motsu_expect("should mint tokens for Bob");

        let alice_balance = contract.sender(alice).balance_of(alice, id);
        let bob_balance = contract.sender(alice).balance_of(bob, id);

        let err = contract
            .sender(alice)
            .transfer(bob, id, one + one)
            .motsu_unwrap_err();
        assert!(matches!(err, Error::InsufficientBalance(_)));

        assert_eq!(alice_balance, contract.sender(alice).balance_of(alice, id));

        assert_eq!(bob_balance, contract.sender(alice).balance_of(bob, id));
    }

    #[motsu::test]
    fn transfer_from(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;
        let ten = uint!(10_U256);

        contract
            .sender(alice)
            .approve(bob, id, one)
            .motsu_expect("should Alice approves Bob");

        contract
            .sender(alice)
            ._mint(alice, id, ten)
            .motsu_expect("should mint tokens for Alice");

        assert_eq!(ten, contract.sender(alice).balance_of(alice, id));

        contract
            .sender(bob)
            .transfer_from(alice, bob, id, one)
            .motsu_expect("should transfer from Alice to Bob");

        assert_eq!(ten - one, contract.sender(alice).balance_of(alice, id));

        assert_eq!(one, contract.sender(alice).balance_of(bob, id));

        contract.assert_emitted(&Transfer {
            caller: bob,
            sender: alice,
            receiver: bob,
            id,
            amount: one,
        });
    }

    #[motsu::test]
    fn transfer_from_errors_insufficient_balance(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;
        let ten = uint!(10_U256);

        contract
            .sender(alice)
            .approve(bob, id, ten)
            .motsu_expect("should Alice approves Bob");

        contract
            .sender(alice)
            ._mint(alice, id, one)
            .motsu_expect("should mint tokens for Alice");

        assert_eq!(one, contract.sender(alice).balance_of(alice, id));

        let err = contract
            .sender(bob)
            .transfer_from(alice, bob, id, ten)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InsufficientBalance(_)));

        assert_eq!(one, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn transfer_from_errors_invalid_receiver(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        contract
            .sender(alice)
            .approve(bob, id, one)
            .motsu_expect("should Alice approves Bob");

        contract
            .sender(alice)
            ._mint(alice, id, one)
            .motsu_expect("should mint tokens for Alice");

        let err = contract
            .sender(bob)
            .transfer_from(alice, Address::ZERO, id, one)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidReceiver(_)));

        assert_eq!(one, contract.sender(alice).balance_of(alice, id));
    }

    #[motsu::test]
    fn transfer_from_errors_insufficient_allowance(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        contract
            .sender(alice)
            ._mint(alice, id, one)
            .motsu_expect("should mint tokens for Alice");

        let err = contract
            .sender(bob)
            .transfer_from(alice, bob, id, one)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InsufficientAllowance(_)));
    }

    #[motsu::test]
    fn approves_and_reads_allowance(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        let allowance = contract.sender(alice).allowance(alice, bob, id);
        assert_eq!(U256::ZERO, allowance);

        contract
            .sender(alice)
            .approve(bob, id, one)
            .motsu_expect("should Alice approves Bob");

        let current_allowance =
            contract.sender(alice).allowance(alice, bob, id);
        assert_eq!(one, current_allowance);

        contract.assert_emitted(&Approval {
            owner: alice,
            spender: bob,
            id,
            amount: one,
        });
    }

    #[motsu::test]
    fn approve_errors_invalid_spender(
        contract: Contract<Erc6909>,
        alice: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        let err = contract
            .sender(alice)
            .approve(Address::ZERO, id, one)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidSpender(_)));
    }

    #[motsu::test]
    fn approve_errors_invalid_approver(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let id = uint!(2_U256);
        let one = U256::ONE;

        let err = contract
            .sender(alice)
            ._approve(Address::ZERO, bob, id, one, false)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidApprover(_)));
    }

    #[motsu::test]
    fn set_operator_and_reads_operator(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let is_operator = contract.sender(alice).is_operator(alice, bob);
        assert!(!is_operator);

        contract
            .sender(alice)
            .set_operator(bob, true)
            .motsu_expect("should Alice sets Bob as operator");

        let is_operator = contract.sender(alice).is_operator(alice, bob);
        assert!(is_operator);

        contract.assert_emitted(&OperatorSet {
            owner: alice,
            spender: bob,
            approved: true,
        });
    }

    #[motsu::test]
    fn set_operator_errors_invalid_spender(
        contract: Contract<Erc6909>,
        alice: Address,
    ) {
        let err = contract
            .sender(alice)
            .set_operator(Address::ZERO, true)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidSpender(_)));
    }

    #[motsu::test]
    fn set_operator_errors_invalid_approver(
        contract: Contract<Erc6909>,
        alice: Address,
        bob: Address,
    ) {
        let err = contract
            .sender(alice)
            ._set_operator(Address::ZERO, bob, true)
            .motsu_unwrap_err();

        assert!(matches!(err, Error::InvalidApprover(_)));
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc6909 as IErc6909>::interface_id();
        let expected: B32 = 0x0f632fb3.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc6909>, alice: Address) {
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc6909 as IErc6909>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc6909 as IErc165>::interface_id()));

        let fake_interface_id = 0x12345678u32;
        assert!(!contract
            .sender(alice)
            .supports_interface(fake_interface_id.into()));
    }
}
