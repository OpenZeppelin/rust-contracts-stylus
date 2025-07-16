//! Implementation of the ERC-20 token standard.
//!
//! We have followed general `OpenZeppelin` Contracts guidelines: functions
//! revert instead of returning `false` on failure. This behavior is
//! nonetheless conventional and does not conflict with the expectations of
//! [`Erc20`] applications.

use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, FixedBytes, U256};
use openzeppelin_stylus_proc::interface_id;
pub use sol::*;
use stylus_sdk::{
    call::MethodError,
    evm, msg,
    prelude::*,
    storage::{StorageMap, StorageU256},
};

// pub mod extensions;
// pub mod interface;
// pub mod utils;
use crate::utils::{
    introspection::erc165::IErc165,
    math::storage::{AddAssignChecked, AddAssignUnchecked, SubAssignUnchecked},
};
#[cfg_attr(coverage_nightly, coverage(off))]
mod sol {
    use alloy_sol_macro::sol;

    sol! {
        /// Emitted when `value` tokens are moved from one account (`from`) to
        /// another (`to`).
        ///
        /// Note that `value` may be zero.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Transfer(address indexed from, address indexed to, uint256 value);
        /// Emitted when the allowance of a `spender` for an `owner` is set by a
        /// call to `approve`. `value` is the new allowance.
        #[derive(Debug)]
        #[allow(missing_docs)]
        event Approval(address indexed owner, address indexed spender, uint256 value);
    }

    sol! {
        /// Indicates an error related to the current `balance` of `sender`. Used
        /// in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        /// * `balance` - Current balance for the interacting account.
        /// * `needed` - Minimum amount required to perform a transfer.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InsufficientBalance(address sender, uint256 balance, uint256 needed);
        /// Indicates a failure with the token `sender`. Used in transfers.
        ///
        /// * `sender` - Address whose tokens are being transferred.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InvalidSender(address sender);
        /// Indicates a failure with the token `receiver`. Used in transfers.
        ///
        /// * `receiver` - Address to which the tokens are being transferred.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InvalidReceiver(address receiver);
        /// Indicates a failure with the `spender`’s `allowance`. Used in
        /// transfers.
        ///
        /// * `spender` - Address that may be allowed to operate on tokens without
        /// being their owner.
        /// * `allowance` - Amount of tokens a `spender` is allowed to operate
        /// with.
        /// * `needed` - Minimum amount required to perform a transfer.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InsufficientAllowance(address spender, uint256 allowance, uint256 needed);
        /// Indicates a failure with the `spender` to be approved. Used in
        /// approvals.
        ///
        /// * `spender` - Address that may be allowed to operate on tokens without
        /// being their owner.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InvalidSpender(address spender);

        /// Indicates a failure with the `approver` of a token to be approved. Used in approvals.
        /// approver Address initiating an approval operation.
        ///
        /// * `approver` - Address initiating an approval operation.
        #[derive(Debug)]
        #[allow(missing_docs)]
        error ERC20InvalidApprover(address approver);

    }
}

/// An [`Erc20`] error defined as described in [ERC-6093].
///
/// [ERC-6093]: https://eips.ethereum.org/EIPS/eip-6093
#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates an error related to the current balance of `sender`. Used in
    /// transfers.
    InsufficientBalance(ERC20InsufficientBalance),
    /// Indicates a failure with the token `sender`. Used in transfers.
    InvalidSender(ERC20InvalidSender),
    /// Indicates a failure with the token `receiver`. Used in transfers.
    InvalidReceiver(ERC20InvalidReceiver),
    /// Indicates a failure with the `spender`’s `allowance`. Used in
    /// transfers.
    InsufficientAllowance(ERC20InsufficientAllowance),
    /// Indicates a failure with the `spender` to be approved. Used in
    /// approvals.
    InvalidSpender(ERC20InvalidSpender),
    /// Indicates a failure with the `approver` of a token to be approved. Used
    /// in approvals. approver Address initiating an approval operation.
    InvalidApprover(ERC20InvalidApprover),
}

impl MethodError for Error {
    fn encode(self) -> alloc::vec::Vec<u8> {
        self.into()
    }
}

/// Storage trait for the ERC-20 token.
pub trait Erc20Storage {
    /// Maps users to balances.
    fn balances(&self) -> &StorageMap<Address, StorageU256>;
    /// Maps users to balances.
    fn balances_mut(&mut self) -> &mut StorageMap<Address, StorageU256>;
    /// Maps users to a mapping of each spender's allowance.
    fn allowances(
        &self,
    ) -> &StorageMap<Address, StorageMap<Address, StorageU256>>;
    /// Maps users to a mapping of each spender's allowance.
    fn allowances_mut(
        &mut self,
    ) -> &mut StorageMap<Address, StorageMap<Address, StorageU256>>;
    /// The total supply of the token.
    fn total_supply(&self) -> &StorageU256;
    /// The total supply of the token.
    fn total_supply_mut(&mut self) -> &mut StorageU256;
}

/// Required interface of an [`Erc20`] compliant contract.
#[interface_id]
pub trait IErc20: Erc20Storage + Erc20Internal {
    /// Returns the number of tokens in existence.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    fn total_supply(&self) -> U256 {
        Erc20Storage::total_supply(self).get()
    }

    /// Returns the number of tokens owned by `account`.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `account` - Account to get balance from.
    fn balance_of(&self, account: Address) -> U256 {
        Erc20Storage::balances(self).get(account)
    }

    /// Moves a `value` amount of tokens from the caller's account to `to`.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `to` address is [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the caller doesn't have a balance
    ///   of at least `value`.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        let from = msg::sender();
        Erc20Internal::_transfer(self, from, to, value)?;
        Ok(true)
    }

    /// Returns the remaining number of tokens that `spender` will be allowed
    /// to spend on behalf of `owner` through `transfer_from`. This is zero by
    /// default.
    ///
    /// This value changes when `approve` or `transfer_from` are called.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        Erc20Storage::allowances(self).get(owner).get(spender)
    }

    /// Sets a `value` number of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// WARNING: Beware that changing an allowance with this method brings the
    /// risk that someone may use both the old and the new allowance by
    /// unfortunate transaction ordering. One possible solution to mitigate
    /// this race condition is to first reduce the `spender`'s allowance to 0
    /// and set the desired value afterwards:
    /// <https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729>
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `spender` - Account that will spend the tokens.
    /// * `value` - Number of tokens the spender is allowed to spend.
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
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        let owner = msg::sender();
        Erc20Internal::_approve(self, owner, spender, value, true)
    }

    /// Moves a `value` number of tokens from `from` to `to` using the
    /// allowance mechanism. `value` is then deducted from the caller's
    /// allowance.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    ///
    /// NOTE: If `value` is the maximum [`U256::MAX`], the allowance is not
    /// updated on `transfer_from`. This is semantically equivalent to
    /// an infinite approval.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - Number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `from` address is [`Address::ZERO`].
    /// * [`Error::InvalidReceiver`] - If the `to` address is [`Address::ZERO`].
    /// * [`Error::InsufficientAllowance`] - If not enough allowance is
    ///   available.
    /// * [`Error::InsufficientBalance`] - If the `from` address doesn't have
    ///   enough tokens, then the error
    ///  is returned.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        let spender = msg::sender();
        Erc20Internal::_spend_allowance(self, from, spender, value)?;
        Erc20Internal::_transfer(self, from, to, value)?;
        Ok(true)
    }
}

/// Internal shared functions for the ERC-20 token.
pub trait Erc20Internal: Erc20Storage {
    /// Sets a `value` number of tokens as the allowance of `spender` over the
    /// caller's tokens.
    ///
    /// Returns a boolean value indicating whether the operation succeeded.
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account that owns the tokens.
    /// * `spender` - Account that will spend the tokens.
    /// * `emit_event` - Emit an [`Approval`] event flag.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSpender`] - If the `spender` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Approval`].
    fn _approve(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
        emit_event: bool,
    ) -> Result<bool, Vec<u8>> {
        if owner.is_zero() {
            return Err(Error::InvalidApprover(ERC20InvalidApprover {
                approver: Address::ZERO,
            })
            .into());
        }

        if spender.is_zero() {
            return Err(Error::InvalidSpender(ERC20InvalidSpender {
                spender: Address::ZERO,
            })
            .into());
        }

        Erc20Storage::allowances_mut(self).setter(owner).insert(spender, value);
        if emit_event {
            evm::log(Approval { owner, spender, value });
        }
        Ok(true)
    }

    /// Internal implementation of transferring tokens between two accounts.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `from` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - The number of tokens to transfer.
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
    fn _transfer(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        if from.is_zero() {
            return Err(Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            })
            .into());
        }
        if to.is_zero() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into());
        }

        self._update(from, to, value)?;

        Ok(())
    }

    /// Creates a `value` amount of tokens and assigns them to `account`,
    /// by transferring it from [`Address::ZERO`].
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidReceiver`] - If the `account` address is
    ///   [`Address::ZERO`].
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    ///
    /// # Panics
    ///
    /// * If `total_supply` exceeds [`U256::MAX`].
    fn _mint(&mut self, account: Address, value: U256) -> Result<(), Vec<u8>> {
        if account.is_zero() {
            return Err(Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO,
            })
            .into());
        }
        self._update(Address::ZERO, account, value)
    }

    /// Transfers a `value` amount of tokens from `from` to `to`, or
    /// alternatively mints (or burns) if `from` (or `to`) is the zero address.
    ///
    /// All customizations to transfers, mints, and burns should be done by
    /// using this function.
    ///
    /// # Arguments
    ///
    /// * `from` - Owner's address.
    /// * `to` - Recipient's address.
    /// * `value` - Amount to be transferred.
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
    /// * If `total_supply` exceeds [`U256::MAX`]. It may happen during `mint`
    ///   operation.
    fn _update(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        if from.is_zero() {
            // Mint operation. Overflow check required: the rest of the code
            // assumes that `total_supply` never overflows.
            Erc20Storage::total_supply_mut(self).add_assign_checked(
                value,
                "should not exceed `U256::MAX` for `total_supply`",
            );
        } else {
            let from_balance = Erc20Storage::balances(self).get(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance(
                    ERC20InsufficientBalance {
                        sender: from,
                        balance: from_balance,
                        needed: value,
                    },
                )
                .into());
            }
            // Overflow not possible:
            // `value` <= `from_balance` <= `total_supply`.
            Erc20Storage::balances_mut(self)
                .setter(from)
                .set(from_balance - value);
        }

        if to.is_zero() {
            // Overflow not possible:
            // `value` <= `total_supply` or
            // `value` <= `from_balance` <= `total_supply`.
            Erc20Storage::total_supply_mut(self).sub_assign_unchecked(value);
        } else {
            // Overflow not possible:
            // `balance_to` + `value` is at most `total_supply`,
            // which fits into a `U256`.
            Erc20Storage::balances_mut(self)
                .setter(to)
                .add_assign_unchecked(value);
        }

        evm::log(Transfer { from, to, value });

        Ok(())
    }

    /// Destroys a `value` amount of tokens from `account`,
    /// lowering the total supply.
    ///
    /// # Arguments
    ///
    /// * `account` - Owner's address.
    /// * `value` - Amount to be burnt.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidSender`] - If the `from` address is [`Address::ZERO`].
    /// * [`Error::InsufficientBalance`] - If the `from` address doesn't have
    ///   enough tokens.
    ///
    /// # Events
    ///
    /// * [`Transfer`].
    fn _burn(&mut self, account: Address, value: U256) -> Result<(), Vec<u8>> {
        if account == Address::ZERO {
            return Err(Error::InvalidSender(ERC20InvalidSender {
                sender: Address::ZERO,
            })
            .into());
        }
        self._update(account, Address::ZERO, value)
    }

    /// Updates `owner`'s allowance for `spender` based on spent `value`.
    ///
    /// Does not update the allowance value in the case of infinite allowance.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `owner` - Account to transfer tokens from.
    /// * `to` - Account to transfer tokens to.
    /// * `value` - The number of tokens to transfer.
    ///
    /// # Errors
    ///
    /// * [`Error::InsufficientAllowance`] - If not enough allowance is
    ///   available.
    ///
    /// # Events
    ///
    /// * [`Approval`].
    fn _spend_allowance(
        &mut self,
        owner: Address,
        spender: Address,
        value: U256,
    ) -> Result<(), Vec<u8>> {
        let current_allowance =
            Erc20Storage::allowances(self).get(owner).get(spender);
        if current_allowance != U256::MAX {
            if current_allowance < value {
                return Err(Error::InsufficientAllowance(
                    ERC20InsufficientAllowance {
                        spender,
                        allowance: current_allowance,
                        needed: value,
                    },
                )
                .into());
            }

            self._approve(owner, spender, current_allowance - value, false)?;
        }

        Ok(())
    }
}

/// Storage for the ERC-20 token.
#[storage]
pub struct Erc20 {
    /// Maps users to balances.
    balances: StorageMap<Address, StorageU256>,
    /// Maps users to a mapping of each spender's allowance.
    allowances: StorageMap<Address, StorageMap<Address, StorageU256>>,
    /// The total supply of the token.
    total_supply: StorageU256,
}

impl Erc20Storage for Erc20 {
    fn balances(&self) -> &StorageMap<Address, StorageU256> {
        &self.balances
    }

    fn balances_mut(&mut self) -> &mut StorageMap<Address, StorageU256> {
        &mut self.balances
    }

    fn allowances(
        &self,
    ) -> &StorageMap<Address, StorageMap<Address, StorageU256>> {
        &self.allowances
    }

    fn allowances_mut(
        &mut self,
    ) -> &mut StorageMap<Address, StorageMap<Address, StorageU256>> {
        &mut self.allowances
    }

    fn total_supply(&self) -> &StorageU256 {
        &self.total_supply
    }

    fn total_supply_mut(&mut self) -> &mut StorageU256 {
        &mut self.total_supply
    }
}

#[public]
#[implements(IErc20)]
impl Erc20 {}

#[public]
impl IErc20 for Erc20 {}

impl Erc20Internal for Erc20 {}

impl IErc165 for Erc20 {
    fn supports_interface(&self, interface_id: FixedBytes<4>) -> bool {
        <Self as IErc20>::interface_id() == interface_id
            || <Self as IErc165>::interface_id() == interface_id
    }
}

#[cfg(test)]
mod tests {
    use core::ops::{Deref, DerefMut};

    use alloy_primitives::{uint, Address};
    use motsu::prelude::*;

    use super::*;

    unsafe impl TopLevelStorage for Erc20 {}

    #[motsu::test]
    fn mint(contract: Contract<Erc20>, alice: Address) {
        let one = uint!(1_U256);

        // Store initial balance & supply.
        let initial_balance = contract.sender(alice).balance_of(alice);
        let initial_supply =
            IErc20::total_supply(contract.sender(alice).deref());

        // Mint action should work.
        let result = Erc20Internal::_mint(
            contract.sender(alice).deref_mut(),
            alice,
            one,
        );
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(
            initial_balance + one,
            contract.sender(alice).balance_of(alice)
        );
        assert_eq!(
            initial_supply + one,
            IErc20::total_supply(contract.sender(alice).deref())
        );

        contract.assert_emitted(&Transfer {
            from: Address::ZERO,
            to: alice,
            value: one,
        });
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `total_supply`"]
    fn update_mint_errors_arithmetic_overflow(
        contract: Contract<Erc20>,
        alice: Address,
    ) {
        let one = uint!(1_U256);
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice));
        assert_eq!(
            U256::ZERO,
            IErc20::total_supply(contract.sender(alice).deref())
        );

        // Initialize state for the test case:
        // Alice's balance as `U256::MAX`.
        contract
            .sender(alice)
            ._mint(alice, U256::MAX)
            .motsu_expect("should mint tokens");
        // Mint action should NOT work:
        // overflow on `total_supply`.
        let _result = contract.sender(alice)._mint(alice, one);
    }

    #[motsu::test]
    fn mint_errors_invalid_receiver(contract: Contract<Erc20>, alice: Address) {
        let receiver = Address::ZERO;
        let one = uint!(1_U256);

        // Store initial balance & supply.
        let initial_balance = contract.sender(alice).balance_of(receiver);
        let initial_supply =
            IErc20::total_supply(contract.sender(alice).deref());

        // Mint action should work.
        let err =
            contract.sender(alice)._mint(receiver, one).motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InvalidReceiver(ERC20InvalidReceiver { receiver }).encode()
        );

        // Check updated balance & supply.
        assert_eq!(
            initial_balance,
            contract.sender(alice).balance_of(receiver)
        );
        assert_eq!(
            initial_supply,
            IErc20::total_supply(contract.sender(alice).deref())
        );
    }

    #[motsu::test]
    #[should_panic = "should not exceed `U256::MAX` for `total_supply`"]
    fn mint_errors_arithmetic_overflow(
        contract: Contract<Erc20>,
        alice: Address,
    ) {
        let one = uint!(1_U256);
        assert_eq!(U256::ZERO, contract.sender(alice).balance_of(alice));
        assert_eq!(
            U256::ZERO,
            IErc20::total_supply(contract.sender(alice).deref())
        );

        // Initialize state for the test case:
        // Alice's balance as [`U256::MAX`].
        contract
            .sender(alice)
            ._mint(alice, U256::MAX)
            .motsu_expect("should mint tokens");
        // Mint action should NOT work -- overflow on `_total_supply`.
        let _result = contract.sender(alice)._mint(alice, one);
    }

    #[motsu::test]
    fn burn(contract: Contract<Erc20>, alice: Address) {
        let one = uint!(1_U256);
        let two = uint!(2_U256);

        // Initialize state for the test case:
        // Alice's balance as `two`.
        contract
            .sender(alice)
            ._mint(alice, two)
            .motsu_expect("should mint tokens");

        // Store initial balance & supply.
        let initial_balance = contract.sender(alice).balance_of(alice);
        let initial_supply =
            IErc20::total_supply(contract.sender(alice).deref());

        // Burn action should work.
        contract.sender(alice)._burn(alice, one).motsu_unwrap();

        // Check updated balance & supply.
        assert_eq!(
            initial_balance - one,
            contract.sender(alice).balance_of(alice)
        );
        assert_eq!(
            initial_supply - one,
            IErc20::total_supply(contract.sender(alice).deref())
        );

        contract.assert_emitted(&Transfer {
            from: alice,
            to: Address::ZERO,
            value: one,
        });
    }

    #[motsu::test]
    fn burn_errors_insufficient_balance(
        contract: Contract<Erc20>,
        alice: Address,
    ) {
        let one = uint!(1_U256);
        let two = uint!(2_U256);

        // Initialize state for the test case:
        // Alice's balance as `one`.
        contract
            .sender(alice)
            ._mint(alice, one)
            .motsu_expect("should mint tokens");

        // Store initial balance & supply.
        let initial_balance = contract.sender(alice).balance_of(alice);
        let initial_supply =
            IErc20::total_supply(contract.sender(alice).deref());

        // Burn action should NOT work - `InsufficientBalance`.
        let err = contract.sender(alice)._burn(alice, two).motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InsufficientBalance(ERC20InsufficientBalance {
                sender: alice,
                needed: two,
                balance: one,
            })
            .encode()
        );

        // Check proper state (before revert).
        assert_eq!(initial_balance, contract.sender(alice).balance_of(alice));
        assert_eq!(
            initial_supply,
            IErc20::total_supply(contract.sender(alice).deref())
        );
    }

    #[motsu::test]
    fn transfer(contract: Contract<Erc20>, alice: Address, bob: Address) {
        let one = uint!(1_U256);

        // Initialize state for the test case:
        //  Alice's & Bob's balance as `one`.
        contract
            .sender(alice)
            ._mint(alice, one)
            .motsu_expect("should mint tokens");
        contract
            .sender(alice)
            ._mint(bob, one)
            .motsu_expect("should mint tokens");

        // Store initial balance & supply.
        let initial_alice_balance = contract.sender(alice).balance_of(alice);
        let initial_bob_balance = contract.sender(alice).balance_of(bob);
        let initial_supply =
            IErc20::total_supply(contract.sender(alice).deref());

        // Transfer action should work.
        let result = contract.sender(alice).transfer(bob, one);
        assert!(result.is_ok());

        // Check updated balance & supply.
        assert_eq!(
            initial_alice_balance - one,
            contract.sender(alice).balance_of(alice)
        );
        assert_eq!(
            initial_bob_balance + one,
            contract.sender(alice).balance_of(bob)
        );
        assert_eq!(
            initial_supply,
            IErc20::total_supply(contract.sender(alice).deref())
        );

        contract.assert_emitted(&Transfer { from: alice, to: bob, value: one });
    }

    #[motsu::test]
    fn transfer_errors_insufficient_balance(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let one = uint!(1_U256);
        let two = uint!(2_U256);

        // Initialize state for the test case:
        // Alice's & Bob's balance as `one`.
        contract
            .sender(alice)
            ._mint(alice, one)
            .motsu_expect("should mint tokens");
        contract
            .sender(alice)
            ._mint(bob, one)
            .motsu_expect("should mint tokens");

        // Store initial balance & supply.
        let initial_alice_balance = contract.sender(alice).balance_of(alice);
        let initial_bob_balance = contract.sender(alice).balance_of(bob);
        let initial_supply =
            IErc20::total_supply(contract.sender(alice).deref());

        // Transfer action should NOT work - `InsufficientBalance`.
        let err = contract.sender(alice).transfer(bob, two).motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InsufficientBalance(ERC20InsufficientBalance {
                sender: alice,
                needed: two,
                balance: one,
            })
            .encode()
        );

        // Check proper state (before revert).
        assert_eq!(
            initial_alice_balance,
            contract.sender(alice).balance_of(alice)
        );
        assert_eq!(initial_bob_balance, contract.sender(alice).balance_of(bob));
        assert_eq!(
            initial_supply,
            IErc20::total_supply(contract.sender(alice).deref())
        );
    }

    #[motsu::test]
    fn transfer_from(contract: Contract<Erc20>, alice: Address, bob: Address) {
        // Alice approves Bob.
        let one = uint!(1_U256);
        contract.sender(alice).approve(bob, one).motsu_unwrap();

        // Mint some tokens for Alice.
        let two = uint!(2_U256);
        contract.sender(alice)._mint(alice, two).motsu_unwrap();
        assert_eq!(two, contract.sender(alice).balance_of(alice));

        contract.sender(bob).transfer_from(alice, bob, one).motsu_unwrap();

        assert_eq!(one, contract.sender(alice).balance_of(alice));
        assert_eq!(one, contract.sender(alice).balance_of(bob));
        assert_eq!(U256::ZERO, contract.sender(alice).allowance(alice, bob));

        contract.assert_emitted(&Transfer { from: alice, to: bob, value: one });
    }

    #[motsu::test]
    fn error_when_transfer_with_insufficient_balance(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        // Alice approves Bob.
        let one = uint!(1_U256);
        contract.sender(alice).approve(bob, one).motsu_unwrap();

        let err = contract
            .sender(bob)
            .transfer_from(alice, bob, one)
            .motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InsufficientBalance(ERC20InsufficientBalance {
                sender: alice,
                needed: one,
                balance: U256::ZERO,
            })
            .encode()
        );
    }

    #[motsu::test]
    fn error_when_transfer_to_invalid_receiver(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        // Alice approves Bob.
        let one = uint!(1_U256);
        contract.sender(alice).approve(bob, one).motsu_unwrap();

        let err = contract
            .sender(bob)
            .transfer_from(alice, Address::ZERO, one)
            .motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InvalidReceiver(ERC20InvalidReceiver {
                receiver: Address::ZERO
            })
            .encode()
        );
    }

    #[motsu::test]
    fn errors_when_transfer_with_insufficient_allowance(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        // Mint some tokens for Alice.
        let one = uint!(1_U256);
        contract.sender(alice)._mint(alice, one).motsu_unwrap();
        assert_eq!(one, contract.sender(alice).balance_of(alice));

        let err = contract
            .sender(alice)
            .transfer_from(alice, bob, one)
            .motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InsufficientAllowance(ERC20InsufficientAllowance {
                spender: alice,
                needed: one,
                allowance: U256::ZERO,
            })
            .encode()
        );
    }

    #[motsu::test]
    fn approves_and_reads_allowance(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let allowance = contract.sender(alice).allowance(alice, bob);
        assert_eq!(U256::ZERO, allowance);

        let one = uint!(1_U256);
        contract.sender(alice).approve(bob, one).motsu_unwrap();
        let allowance = contract.sender(alice).allowance(alice, bob);
        assert_eq!(one, allowance);

        contract.assert_emitted(&Approval {
            owner: alice,
            spender: bob,
            value: one,
        });
    }

    #[motsu::test]
    fn error_when_approve_for_invalid_spender(
        contract: Contract<Erc20>,
        alice: Address,
    ) {
        // alice approves [`Address::ZERO`].
        let one = uint!(1_U256);
        let err = contract
            .sender(alice)
            .approve(Address::ZERO, one)
            .motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InvalidSpender(ERC20InvalidSpender {
                spender: Address::ZERO
            })
            .encode()
        );
    }

    #[motsu::test]
    fn error_when_invalid_approver(
        contract: Contract<Erc20>,
        alice: Address,
        bob: Address,
    ) {
        let one = uint!(1_U256);
        let err = contract
            .sender(alice)
            ._approve(Address::ZERO, bob, one, false)
            .motsu_unwrap_err();
        assert_eq!(
            err,
            Error::InvalidApprover(ERC20InvalidApprover {
                approver: Address::ZERO
            })
            .encode()
        );
    }

    #[motsu::test]
    fn interface_id() {
        let actual = <Erc20 as IErc20>::interface_id();
        let expected: FixedBytes<4> = 0x36372b07_u32.into();
        assert_eq!(actual, expected);
    }

    #[motsu::test]
    fn supports_interface(contract: Contract<Erc20>, alice: Address) {
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc20 as IErc20>::interface_id()));
        assert!(contract
            .sender(alice)
            .supports_interface(<Erc20 as IErc165>::interface_id()));

        let fake_interface_id = 0x12345678u32;
        assert!(!contract
            .sender(alice)
            .supports_interface(fake_interface_id.into()));
    }
}
